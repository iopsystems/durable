# Deterministic Simulation Testing (DST) for Durable

## Executive Summary

Durable already has strong foundations for deterministic execution — the transaction
replay system records all non-deterministic operations (time, randomness, I/O, SQL)
into an event log and replays them on restart. DST would extend this by allowing
the entire worker + database + network environment to be simulated in a single
process with controlled scheduling, enabling exhaustive testing of failure scenarios.

This document covers what would be needed and proposes concrete hooks to add.

---

## 1. Current Determinism Landscape

### What's already deterministic
- **WASM execution**: WebAssembly is inherently deterministic. All non-determinism
  enters through host function imports.
- **Transaction replay**: Every host call (clock, random, HTTP, SQL) is wrapped in
  `maybe_do_transaction_sync()` / `do_transaction()` with a label. On replay, cached
  results are returned instead of re-executing.
- **Label validation**: `try_enter_cached()` validates that replay labels match,
  detecting execution divergence immediately.
- **Insecure seed**: `insecure_seed()` is already deterministic — it hashes
  `(task_id, task_name)`.
- **Single-threaded WASM**: Each task runs in a single WASM instance with no
  internal concurrency.

### Sources of non-determinism that matter for DST
| Source | Location | Current handling |
|--------|----------|-----------------|
| Wall clock | `plugin/wasi/clocks.rs` | Recorded in event log |
| Monotonic clock | `plugin/wasi/clocks.rs` | Recorded in event log |
| `getrandom` | `plugin/wasi/random.rs` | Recorded in event log |
| `rand::rng()` | `plugin/wasi/random.rs` | Recorded in event log |
| HTTP requests | `plugin/durable/http.rs` | Recorded in event log |
| SQL queries | `plugin/durable/sql/` | Recorded + DB transaction |
| Task scheduling order | `worker.rs` event loop | **Not controlled** |
| Timer wakeups | `plugin/wasi/io.rs` | Suspend/resume via DB |
| Worker leader election | `worker.rs` heartbeats | **Not controlled** |
| PostgreSQL LISTEN/NOTIFY ordering | `event/` | **Not controlled** |
| Tokio task scheduling | Tokio runtime | **Not controlled** |
| Multiple workers racing for tasks | `worker.rs` SQL claims | **Not controlled** |

The first group (recorded operations) is already handled by replay. DST needs to
control the second group — scheduling, timing of wakeups, inter-worker coordination,
and database behavior.

---

## 2. DST Approaches Considered

### Option A: Trait abstraction layer (recommended starting point)
Introduce traits for the major non-deterministic subsystems and provide simulated
implementations. This is the approach used by FoundationDB (from day one) and
Resonate.

**Pros**: Surgical, composable, no external dependencies, works with existing test infra.
**Cons**: Requires some refactoring; third-party crates bypass the abstraction.

### Option B: libc-level interposition (madsim / mad-turmoil)
Override `clock_gettime`, `getrandom`, etc. at the libc level so even third-party
code sees simulated values.

**Pros**: Catches all non-determinism including transitive deps.
**Cons**: Fragile, platform-specific, doesn't help with database simulation.

### Option C: Deterministic hypervisor (Antithesis)
Run the entire system (workers + PostgreSQL) inside a deterministic VM.

**Pros**: Zero code changes required; tests the real system.
**Cons**: Requires Antithesis platform (commercial); not self-hostable.

### Option D: State machine approach (Polar Signals)
Refactor all components into explicit state machines with `tick()` / `receive()`
interfaces and a central message bus.

**Pros**: Cleanest simulation model; trivial fault injection.
**Cons**: Major architectural rewrite; high cognitive overhead.

### Recommendation
**Start with Option A** (trait abstractions + hooks), with the door open to
supplement with Option B for third-party code and Option C for end-to-end
validation. Option A gives the best ROI for durable's architecture because
the runtime already centralizes non-determinism through host function calls.

---

## 3. Proposed Architecture

### 3.1 Core Abstraction: `SimulationContext` trait

```rust
/// Provides hooks for all non-deterministic operations in the runtime.
/// The default implementation uses real time, real I/O, etc.
/// A DST implementation controls all of these.
pub trait SimulationContext: Send + Sync + 'static {
    /// Current wall clock time.
    fn wall_clock_now(&self) -> std::time::SystemTime;

    /// Current monotonic time (as UTC for cross-host consistency).
    fn monotonic_now(&self) -> chrono::DateTime<chrono::Utc>;

    /// Fill buffer with random bytes.
    fn getrandom(&self, buf: &mut [u8]) -> Result<(), getrandom::Error>;

    /// Fill buffer with insecure random bytes.
    fn insecure_random_fill(&self, buf: &mut [u8]);

    /// Sleep for the given duration. In simulation, this advances
    /// the simulated clock instead of actually sleeping.
    async fn sleep(&self, duration: std::time::Duration);

    /// Perform an HTTP request. In simulation, this can return
    /// canned responses, inject errors, or simulate latency.
    async fn http_execute(
        &self,
        client: &reqwest::Client,
        request: reqwest::Request,
    ) -> Result<reqwest::Response, reqwest::Error>;
}
```

This trait would be stored in `SharedState` and threaded through to every
host function implementation.

### 3.2 Database Abstraction: `DatabaseBackend` trait

This is the hardest piece. The runtime is deeply coupled to `sqlx::PgPool`.

```rust
#[async_trait]
pub trait DatabaseBackend: Send + Sync + 'static {
    /// Acquire a connection (real or simulated).
    async fn acquire(&self) -> anyhow::Result<Box<dyn DatabaseConnection>>;

    /// Execute a query that returns rows.
    async fn fetch(
        &self,
        query: &str,
        params: &[SqlValue],
    ) -> anyhow::Result<Vec<SqlRow>>;

    /// Execute a query that modifies data.
    async fn execute(
        &self,
        query: &str,
        params: &[SqlValue],
    ) -> anyhow::Result<u64>; // rows affected

    /// Begin a transaction.
    async fn begin(&self) -> anyhow::Result<Box<dyn DatabaseTransaction>>;
}
```

**Pragmatic alternative**: Rather than abstracting the full database, focus on
abstracting just the _internal_ database operations (event log, task state
management, worker registration) behind a `TaskStore` trait:

```rust
#[async_trait]
pub trait TaskStore: Send + Sync + 'static {
    /// Claim a task for this worker.
    async fn claim_task(&self, worker_id: i64, task_id: i64) -> anyhow::Result<bool>;

    /// Load event history for a task.
    async fn load_events(&self, task_id: i64) -> anyhow::Result<Vec<RecordedEvent>>;

    /// Record an event.
    async fn record_event(
        &self,
        task_id: i64,
        index: i32,
        label: &str,
        value: serde_json::Value,
    ) -> anyhow::Result<()>;

    /// Update task state (complete, failed, suspended).
    async fn set_task_state(
        &self,
        task_id: i64,
        state: TaskState,
    ) -> anyhow::Result<()>;

    /// Check if this worker still owns the task.
    async fn is_task_owner(
        &self,
        task_id: i64,
        worker_id: i64,
    ) -> anyhow::Result<bool>;
}
```

This is more tractable and still enables DST for the core scheduling and
recovery logic. User-facing SQL (via `durable-sqlx`) would still need a real
database or a separate mock.

### 3.3 Scheduler Control: `SimulatedScheduler`

```rust
pub trait TaskScheduler: Send + Sync + 'static {
    /// Called when the worker needs to decide which task to run next.
    /// In simulation, this controls execution order deterministically.
    async fn next_task(&self) -> anyhow::Result<Option<TaskData>>;

    /// Called when a task suspends. In simulation, this can
    /// immediately wake it or delay based on the seed.
    async fn on_task_suspend(
        &self,
        task_id: i64,
        wakeup_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> anyhow::Result<()>;
}
```

### 3.4 Integration with `WorkerBuilder`

```rust
impl WorkerBuilder {
    // Existing methods...

    /// Set a custom simulation context for DST.
    /// When set, the runtime uses this for all time, random, and I/O operations.
    pub fn simulation_context(
        mut self,
        ctx: Arc<dyn SimulationContext>,
    ) -> Self { ... }

    /// Set a custom task store for DST.
    /// When set, the runtime uses this instead of PostgreSQL for internal state.
    pub fn task_store(
        mut self,
        store: Arc<dyn TaskStore>,
    ) -> Self { ... }
}
```

---

## 4. What Needs to Change in Durable

### 4.1 Minimal hooks (low effort, high value)

These changes add DST hooks without breaking existing behavior:

1. **`SimulationContext` trait + `RealContext` default impl**
   - Add to `durable-runtime/src/simulation.rs`
   - Store `Arc<dyn SimulationContext>` in `SharedState`
   - Wire into clock, random, and HTTP host functions
   - Files affected: `worker.rs`, `plugin/wasi/clocks.rs`, `plugin/wasi/random.rs`,
     `plugin/durable/http.rs`, `plugin/wasi/io.rs`

2. **`EventSource` is already a trait** — no changes needed for event injection.

3. **`reqwest::Client` is already injectable** via `WorkerBuilder::client()` —
   HTTP mocking can use `wiremock` or similar without runtime changes.

4. **Add `WorkerBuilder::simulation_context()`** method.

### 4.2 Medium effort (adds significant DST capability)

5. **`TaskStore` trait** abstracting internal DB operations
   - Extract all SQL in `task.rs` and `worker.rs` into a trait
   - Default impl wraps `sqlx::PgPool`
   - Simulated impl uses in-memory state
   - This is the biggest refactor but unlocks database-free testing

6. **Single-threaded runtime mode**
   - Ensure the worker can run on `tokio::runtime::Builder::new_current_thread()`
   - May already work but needs verification and testing

7. **Deterministic task ID generation**
   - Currently auto-generated by PostgreSQL sequences
   - DST needs predictable IDs for reproducibility
   - Add a hook or use seeded ID generation in simulation mode

### 4.3 Advanced (full DST framework)

8. **Multi-worker simulation**
   - Run multiple `Worker` instances in-process sharing simulated state
   - Simulate worker crashes, network partitions, split-brain scenarios
   - Requires `TaskStore` to support concurrent access with configurable conflict behavior

9. **Fault injection hooks**
   - Database connection drops mid-transaction
   - Worker death during task execution
   - Duplicate event delivery
   - Clock skew between workers

10. **Seed-based test runner**
    - Single seed determines all random choices (scheduling, faults, timing)
    - Failed test prints seed for reproduction
    - CI runs thousands of seeds

---

## 5. Implementation Roadmap

### Phase 1: SimulationContext hooks (1-2 weeks of effort)
- Define `SimulationContext` trait
- Add `RealSimulationContext` as the default
- Wire into clock/random/HTTP host functions
- Add `WorkerBuilder::simulation_context()`
- Write a basic DST test that controls time and randomness

### Phase 2: TaskStore abstraction (2-4 weeks)
- Define `TaskStore` trait
- Extract SQL operations into `PgTaskStore`
- Implement `InMemoryTaskStore`
- Wire into worker and task execution
- Write DST tests for task scheduling, replay, and recovery

### Phase 3: Multi-worker simulation (2-4 weeks)
- Shared in-memory `TaskStore` with locking
- Simulated worker lifecycle (start, crash, restart)
- Fault injection at the store and context layers
- Seed-based test runner with CI integration

### Phase 4: User-facing DST hooks
- Expose `SimulationContext` and `TaskStore` traits publicly
- Document how downstream projects can build DST for systems that use durable
- Provide example DST test harness

---

## 6. Key Design Decisions

### Should `SimulationContext` be a trait or a set of traits?
A single trait is simpler and ensures the simulation is coherent (e.g., time
and sleep are coupled). Multiple traits would be more composable but harder to
keep consistent. **Recommendation**: Start with a single trait, split later
if needed.

### Should the database abstraction be at the SQL level or the operation level?
SQL-level abstraction (mock `PgPool`) is closer to reality but requires
reimplementing PostgreSQL semantics. Operation-level abstraction (`TaskStore`)
is easier to implement and sufficient for testing core scheduling/replay logic.
**Recommendation**: Operation-level (`TaskStore`) for internal state; real
PostgreSQL for user-facing SQL tests.

### Should DST use `cfg` features or runtime dispatch?
Runtime dispatch (`dyn Trait`) adds a vtable call per operation but keeps
the code unified. Feature flags create divergent code paths that are hard
to maintain. **Recommendation**: Runtime dispatch. The overhead is negligible
compared to WASM execution and database I/O.

### How to handle `tokio::spawn` and concurrency?
The worker uses `JoinSet` for concurrent task execution. In simulation mode,
these should run on a single-threaded Tokio runtime with `tokio::time::pause()`
to control time. The scheduler would deterministically choose which task
progresses. This requires no code changes if the worker already works on
`current_thread` runtime.

---

## 7. Existing Extension Points (No Changes Needed)

These can be leveraged today for partial DST:

| Extension point | How to use for DST |
|---|---|
| `EventSource` trait | Inject events in deterministic order |
| `WorkerBuilder::client()` | Provide mock HTTP client |
| `WorkerBuilder::plugin()` | Override any WASI or durable host function |
| `WorkerBuilder::wasmtime_config()` | Configure deterministic compilation |
| `Plugin` trait | Replace clock/random/HTTP implementations entirely |
| `sqlx::test` infrastructure | Fresh database per test |

The `Plugin` trait is particularly powerful — a custom plugin could override
the default WASI clock/random implementations with deterministic ones by
re-adding the same WIT imports with different host functions. However, this
would fight with the default `DurablePlugin` which adds all imports. A
cleaner approach is the `SimulationContext` indirection proposed above.

---

## 8. References

- [FoundationDB: Testing Distributed Systems w/ Deterministic Simulation](https://apple.github.io/foundationdb/testing.html)
- [S2: Deterministic Simulation Testing for Async Rust](https://s2.dev/blog/dst) — uses mad-turmoil
- [Polar Signals: DST in Rust](https://www.polarsignals.com/blog/posts/2025/07/08/dst-rust) — state machine approach
- [TigerBeetle: Vortex + DST](https://tigerbeetle.com/blog/2025-02-13-a-descent-into-the-vortex/)
- [Antithesis: Deterministic Hypervisor](https://antithesis.com)
- [madsim](https://github.com/madsim-rs/madsim) — Tokio-compatible simulation runtime
- [turmoil](https://github.com/tokio-rs/turmoil) — network simulation for Tokio
- [Resonate: DST for Durable Execution](https://blog.resonatehq.io/deterministic-simulation-testing)
