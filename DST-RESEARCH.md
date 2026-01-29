# Deterministic Simulation Testing (DST) for Durable

## Executive Summary

Durable already has strong foundations for deterministic execution — the transaction
replay system records all non-deterministic operations (time, randomness, I/O, SQL)
into an event log and replays them on restart. DST would extend this by providing
**deterministic scheduling control** over the independent components that interact
with PostgreSQL, while keeping PostgreSQL as the real storage backend.

The core idea: don't replace the database — instead, control **when** each component
(heartbeat, leader election, task wakeup, event processing, individual task execution)
gets to run, and in what order they interact with PostgreSQL.

---

## 1. Current Architecture: The Concurrent Components

The `Worker::run()` method launches six concurrent components that all interact
with PostgreSQL independently (`worker.rs:281-343`):

| Component | Function | DB interactions |
|-----------|----------|-----------------|
| **heartbeat** | Updates `worker.heartbeat_at` periodically | `UPDATE worker SET heartbeat_at` |
| **validate_workers** | Deletes dead workers | `DELETE FROM worker WHERE heartbeat expired` |
| **leader** | Wakes suspended tasks whose timers expired | `UPDATE task SET state='ready'` |
| **task_cleanup** | Deletes old completed tasks | `DELETE FROM task WHERE completed_at < ...` |
| **stuck_notify** | Unwedges tasks with stuck notifications | `UPDATE task SET state='ready'` |
| **process_events** | Listens for PG events, claims & spawns tasks | `UPDATE task SET running_on=...` |

Additionally, each **running task** is its own concurrent unit that:
- Enters/exits transactions (`durable.event` inserts)
- Reads/writes `durable.task` state
- May suspend (`UPDATE task SET state='suspended'`)
- May spawn child tasks

All of these run concurrently via `futures_concurrency::Join` and `JoinSet`,
with the tokio scheduler determining which one progresses at any given moment.

### Sources of non-determinism

**Within WASM execution** (already handled by replay):
- Wall/monotonic clock reads → recorded in `durable.event`
- Random bytes → recorded in `durable.event`
- HTTP requests → recorded in `durable.event`
- SQL queries → recorded in `durable.event` + DB transaction

**Between components** (the DST target):
- Which component runs next (tokio scheduling)
- Event arrival order from `PgListener` (LISTEN/NOTIFY)
- Task claim races (`SELECT ... FOR NO KEY UPDATE SKIP LOCKED`)
- Timer jitter in heartbeat (`rand::rng().random_range(...)`)
- `Utc::now()` calls in leader wakeup logic, worker validation
- Which worker claims which task when multiple workers exist
- Order of task completion relative to new task arrival

---

## 2. Approach: Component-Level Scheduling with Real PostgreSQL

### Why keep PostgreSQL?

1. **PostgreSQL is the source of truth** — durable's correctness depends on PG
   transaction semantics, `SKIP LOCKED`, `LISTEN/NOTIFY`, and foreign key cascades.
   Replacing PG with a mock would test a different system.

2. **Existing test infra uses real PG** — `sqlx::test` already provides per-test
   databases. DST builds on this rather than replacing it.

3. **The interesting bugs are in scheduling** — the scenarios DST should find
   involve races between components: worker death mid-transaction, task stolen
   during execution, suspend/wakeup ordering, leader election timing. These all
   involve real SQL semantics.

### What to control

Instead of replacing PostgreSQL, we control the **scheduling of components** that
interact with it. Each component becomes a controllable unit that can only progress
when the DST scheduler permits it.

The key insight: durable's worker is structured as independent async loops joined
together. If we can gate when each loop iteration runs, we control the interleaving
without changing the actual database operations.

---

## 3. Proposed Architecture

### 3.1 `Scheduler` trait

The core abstraction: a scheduler that decides which component runs next.

```rust
/// Controls the execution order of worker components.
///
/// In production, this is a no-op (all components run freely).
/// In DST, this gates each component behind a permit, allowing the
/// test harness to deterministically control interleaving.
pub trait Scheduler: Send + Sync + 'static {
    /// Called by a component before it does work. The scheduler can
    /// delay this arbitrarily to control ordering.
    ///
    /// Returns a guard that is held while the component does its work.
    /// When dropped, the scheduler knows the step completed.
    async fn acquire(&self, component: Component) -> ScheduleGuard;

    /// Notify the scheduler about an event (for logging/assertions).
    fn notify(&self, event: ScheduleEvent);
}

#[derive(Clone, Debug)]
pub enum Component {
    Heartbeat { worker_id: i64 },
    ValidateWorkers { worker_id: i64 },
    Leader { worker_id: i64 },
    TaskCleanup { worker_id: i64 },
    StuckNotify { worker_id: i64 },
    ProcessEvents { worker_id: i64 },
    TaskExecution { worker_id: i64, task_id: i64 },

    /// A task entering/exiting a transaction with the database.
    TaskTransaction { task_id: i64, label: String },
}

#[derive(Clone, Debug)]
pub enum ScheduleEvent {
    TaskClaimed { worker_id: i64, task_id: i64 },
    TaskCompleted { task_id: i64, status: TaskStatus },
    TaskSuspended { task_id: i64, wakeup_at: Option<DateTime<Utc>> },
    WorkerRegistered { worker_id: i64 },
    WorkerDeleted { worker_id: i64 },
    LeaderChanged { new_leader: i64 },
    TransactionRecorded { task_id: i64, index: i32, label: String },
}

/// RAII guard returned by `Scheduler::acquire()`.
/// Dropping it signals the scheduler that the step completed.
pub struct ScheduleGuard { /* ... */ }
```

### 3.2 `Clock` trait

Controls time for the non-WASM parts of the runtime (heartbeats, leader
wakeup checks, suspend margin calculations). WASM-visible time is already
handled by the transaction replay system.

```rust
/// Controls the runtime's view of time.
///
/// In production, this delegates to std/chrono.
/// In DST, this returns controlled values so timer-based logic
/// (heartbeat expiry, suspend wakeup, cleanup age) is deterministic.
pub trait Clock: Send + Sync + 'static {
    /// Current UTC time (replaces `Utc::now()` in runtime code).
    fn now(&self) -> DateTime<Utc>;

    /// Sleep until woken or until a duration passes.
    /// In DST, this completes when the scheduler advances time.
    async fn sleep(&self, duration: Duration);

    /// Sleep until a specific instant.
    /// In DST, this completes when the scheduler advances past the instant.
    async fn sleep_until(&self, deadline: tokio::time::Instant);
}
```

### 3.3 `Entropy` trait

Controls randomness for the non-WASM parts of the runtime. The only place this
matters currently is the heartbeat jitter calculation (`worker.rs:379`).

```rust
/// Controls randomness for runtime-internal decisions.
///
/// WASM-visible randomness is already recorded by the transaction system.
/// This only covers randomness used in the worker infrastructure itself.
pub trait Entropy: Send + Sync + 'static {
    /// Generate a random value in the given range.
    fn random_range(&self, range: std::ops::Range<u128>) -> u128;
}
```

### 3.4 Where hooks get inserted

Each worker component loop gets a `scheduler.acquire()` call at the top of
its iteration:

```rust
// Before (heartbeat loop body):
async fn heartbeat(shared: Arc<SharedState>, worker_id: i64) -> anyhow::Result<()> {
    // ... setup ...
    'outer: loop {
        tokio::select! { /* sleep or shutdown */ }

        let record = sqlx::query!(/* UPDATE heartbeat */)
            .fetch_optional(&shared.pool)
            .await?;
        // ...
    }
}

// After:
async fn heartbeat(shared: Arc<SharedState>, worker_id: i64) -> anyhow::Result<()> {
    // ... setup ...
    'outer: loop {
        tokio::select! { /* sleep or shutdown */ }

        let _guard = shared.scheduler.acquire(Component::Heartbeat {
            worker_id
        }).await;

        let record = sqlx::query!(/* UPDATE heartbeat */)
            .fetch_optional(&shared.pool)
            .await?;
        // ...
    }
}
```

Similarly for task transaction enter/exit:

```rust
// In TaskState::exit_impl(), before the DB write:
let _guard = self.shared.scheduler.acquire(Component::TaskTransaction {
    task_id: self.task_id(),
    label: txn.label.to_string(),
}).await;
```

And for time reads in the runtime (not WASM):

```rust
// Before:
let now = Utc::now();

// After:
let now = shared.clock.now();
```

### 3.5 Default (production) implementations

```rust
/// No-op scheduler that never blocks.
pub struct NoopScheduler;

#[async_trait]
impl Scheduler for NoopScheduler {
    async fn acquire(&self, _component: Component) -> ScheduleGuard {
        ScheduleGuard(())
    }
    fn notify(&self, _event: ScheduleEvent) {}
}

/// Real clock using system time.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> { Utc::now() }
    async fn sleep(&self, duration: Duration) { tokio::time::sleep(duration).await }
    async fn sleep_until(&self, deadline: Instant) { tokio::time::sleep_until(deadline).await }
}

/// Real entropy using rand.
pub struct SystemEntropy;

impl Entropy for SystemEntropy {
    fn random_range(&self, range: std::ops::Range<u128>) -> u128 {
        rand::rng().random_range(range)
    }
}
```

### 3.6 DST implementations

```rust
pub struct DstScheduler {
    seed: u64,
    rng: Mutex<StdRng>,
    /// Waiters: each component blocks on a oneshot until selected.
    pending: Mutex<Vec<(Component, oneshot::Sender<()>)>>,
    /// Log of all events for post-test assertions.
    events: Mutex<Vec<ScheduleEvent>>,
}

impl DstScheduler {
    pub fn new(seed: u64) -> Self { /* ... */ }

    /// Advance the simulation by one step: pick a random pending
    /// component and allow it to proceed.
    pub async fn step(&self) {
        let mut pending = self.pending.lock().await;
        if pending.is_empty() { return; }
        let idx = self.rng.lock().await.random_range(0..pending.len());
        let (component, sender) = pending.remove(idx);
        tracing::trace!("DST: scheduling {component:?}");
        let _ = sender.send(());
    }

    /// Run until no components are waiting (quiescent state).
    pub async fn run_until_quiescent(&self) { /* ... */ }

    /// Run for N steps.
    pub async fn run_steps(&self, n: usize) { /* ... */ }
}

impl Scheduler for DstScheduler {
    async fn acquire(&self, component: Component) -> ScheduleGuard {
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.push((component, tx));
        rx.await.unwrap();
        ScheduleGuard(())
    }
    // ...
}

pub struct DstClock {
    now: Mutex<DateTime<Utc>>,
    /// Waiters blocked on sleep/sleep_until
    sleepers: Mutex<Vec<(DateTime<Utc>, oneshot::Sender<()>)>>,
}

impl DstClock {
    /// Advance time by the given duration, waking any expired sleepers.
    pub async fn advance(&self, duration: Duration) { /* ... */ }

    /// Advance time to the next sleeper's deadline.
    pub async fn advance_to_next_sleeper(&self) { /* ... */ }
}

pub struct DstEntropy {
    rng: Mutex<StdRng>,
}
```

### 3.7 Integration with `WorkerBuilder`

```rust
pub struct WorkerBuilder {
    // ... existing fields ...
    scheduler: Option<Arc<dyn Scheduler>>,
    clock: Option<Arc<dyn Clock>>,
    entropy: Option<Arc<dyn Entropy>>,
}

impl WorkerBuilder {
    /// Set a custom scheduler for controlling component interleaving.
    pub fn scheduler(mut self, scheduler: Arc<dyn Scheduler>) -> Self {
        self.scheduler = Some(scheduler);
        self
    }

    /// Set a custom clock for controlling time.
    pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    /// Set a custom entropy source for controlling randomness.
    pub fn entropy(mut self, entropy: Arc<dyn Entropy>) -> Self {
        self.entropy = Some(entropy);
        self
    }
}
```

These get stored in `SharedState` and used throughout the worker.

---

## 4. Changes Required in Durable

### 4.1 New files
- `durable-runtime/src/scheduler.rs` — `Scheduler`, `Component`, `ScheduleEvent`,
  `ScheduleGuard`, `NoopScheduler`
- `durable-runtime/src/clock.rs` — `Clock`, `SystemClock`
- `durable-runtime/src/entropy.rs` — `Entropy`, `SystemEntropy`

### 4.2 Modified files

**`worker.rs`** (most changes):
- Add `scheduler`, `clock`, `entropy` to `SharedState`
- Add builder methods for all three
- Insert `scheduler.acquire()` calls at the top of each component loop body:
  - `heartbeat()` — before the heartbeat UPDATE
  - `validate_workers()` — before the DELETE/SELECT
  - `leader()` — before the suspended task wakeup UPDATE
  - `task_cleanup()` — before the cleanup DELETE
  - `stuck_notify()` — before the stuck notification UPDATE
  - `process_events()` — before processing each event
  - `spawn_new_tasks()` — before the task claim SELECT/UPDATE
  - `run_task()` / `run_task_impl()` — around task lifecycle transitions
- Replace `Utc::now()` with `shared.clock.now()` (5 occurrences)
- Replace `tokio::time::sleep`/`sleep_until` with `shared.clock.sleep()`/`sleep_until()`
- Replace `rand::rng().random_range(...)` with `shared.entropy.random_range(...)`
- Add `scheduler.notify()` calls at key state transitions

**`task.rs`**:
- Insert `scheduler.acquire(Component::TaskTransaction { ... })` in `enter_impl()`
  and `exit_impl()` — this gates when a task can record events to the database
- Replace `Utc::now()` in `maybe_do_transaction_sync` path (none currently, but
  relevant for the IO pollable code)

**`plugin/wasi/io.rs`**:
- Replace `Utc::now()` calls (lines 231, 268, 363) with `shared.clock.now()`
- Replace `tokio::time::sleep(delta)` (lines 288, 409) with `shared.clock.sleep(delta)`
- These are the runtime-side time reads used for poll/block decisions

**`plugin/wasi/clocks.rs`**:
- Optionally replace `SystemTime::now()` and `Utc::now()` with `shared.clock.now()`
  in the host function impls. Since these are already recorded by the transaction
  system, this is less critical — but it makes the clock consistent between WASM
  and runtime views, which matters for suspension logic.

**`plugin/mod.rs`**:
- No changes needed to the `Plugin` trait itself.

### 4.3 What does NOT change
- All SQL queries remain exactly as-is — real PostgreSQL executes them
- The `EventSource` trait and `PgEventSource` — already injectable
- WASM compilation and instantiation
- The transaction replay system (`enter`/`exit`/`try_enter_cached`)
- The `Plugin` trait and existing plugins

---

## 5. How DST Tests Would Work

### Basic structure

```rust
#[sqlx::test(migrations = "...")]
async fn test_task_completes_despite_scheduling_chaos(pool: PgPool) {
    let seed = 42u64;
    let scheduler = Arc::new(DstScheduler::new(seed));
    let clock = Arc::new(DstClock::new(Utc::now()));
    let entropy = Arc::new(DstEntropy::new(seed));

    // Build worker with DST hooks
    let mut worker = WorkerBuilder::new(pool.clone())
        .scheduler(scheduler.clone())
        .clock(clock.clone())
        .entropy(entropy.clone())
        .migrate(true)
        .build()
        .await
        .unwrap();

    // Spawn the worker in the background
    let handle = worker.handle();
    let worker_task = tokio::spawn(async move { worker.run().await });

    // Launch a test workflow
    let client = DurableClient::new(pool.clone()).unwrap();
    let program = load_binary(&client, "simple-workflow.wasm").await;
    let task = client.launch("test", &program, &json!(null)).await.unwrap();

    // Drive the simulation: random interleaving of components
    for _ in 0..1000 {
        scheduler.step().await;
        // Occasionally advance time
        if entropy.random_range(0..10) == 0 {
            clock.advance(Duration::from_secs(1)).await;
        }
    }

    // Check the task completed
    let status = task.wait(&client).await.unwrap();
    assert!(status.success());

    handle.shutdown();
    worker_task.await.unwrap().unwrap();
}
```

### Multi-worker test

```rust
#[sqlx::test(migrations = "...")]
async fn test_task_recovery_after_worker_death(pool: PgPool) {
    let seed = 123u64;
    let scheduler = Arc::new(DstScheduler::new(seed));
    let clock = Arc::new(DstClock::new(Utc::now()));

    // Start two workers sharing the same scheduler
    let mut worker1 = WorkerBuilder::new(pool.clone())
        .scheduler(scheduler.clone())
        .clock(clock.clone())
        .build().await.unwrap();
    let handle1 = worker1.handle();

    let mut worker2 = WorkerBuilder::new(pool.clone())
        .scheduler(scheduler.clone())
        .clock(clock.clone())
        .build().await.unwrap();
    let handle2 = worker2.handle();

    let w1 = tokio::spawn(async move { worker1.run().await });
    let w2 = tokio::spawn(async move { worker2.run().await });

    // Launch a task
    let client = DurableClient::new(pool).unwrap();
    let task = client.launch("test", &program, &json!(null)).await.unwrap();

    // Run for a while
    for _ in 0..500 {
        scheduler.step().await;
    }

    // Kill worker 1 mid-execution
    handle1.shutdown();
    w1.await.unwrap().unwrap();

    // Advance time past heartbeat timeout so worker2 detects death
    clock.advance(Duration::from_secs(60)).await;

    // Keep running — worker2 should pick up the task
    for _ in 0..500 {
        scheduler.step().await;
    }

    let status = task.wait(&client).await.unwrap();
    assert!(status.success());
}
```

### Fault injection

The `DstScheduler` can also inject faults:

```rust
impl DstScheduler {
    /// Randomly drop a component's step (simulating a crash/timeout).
    pub fn with_fault_rate(mut self, rate: f64) -> Self { /* ... */ }

    /// Kill a specific worker after N steps.
    pub fn kill_worker_after(&self, worker_id: i64, steps: usize) { /* ... */ }
}
```

### Seed-based reproduction

```rust
#[sqlx::test]
async fn fuzz_scheduling(pool: PgPool) {
    // Run with many seeds, or accept seed from env
    let seed = std::env::var("DST_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| rand::random());

    eprintln!("DST seed: {seed}");

    // ... run test with this seed ...
    // On failure: "DST seed: 12345" is printed, rerun with DST_SEED=12345
}
```

---

## 6. What This Enables for Downstream Users

Users building systems on top of durable can leverage the same hooks:

```rust
// A user's system that orchestrates durable workflows
struct MySystem {
    durable_worker: Worker,
    my_component: MyComponent,
}

// In DST mode, both share the same scheduler
let scheduler = Arc::new(DstScheduler::new(seed));

let worker = WorkerBuilder::new(pool)
    .scheduler(scheduler.clone())
    .build().await?;

// User's component also gates on the scheduler
loop {
    let _guard = scheduler.acquire(Component::Custom("my-component")).await;
    my_component.do_work().await;
}
```

For this to work, `Component` should be extensible:

```rust
pub enum Component {
    // ... built-in variants ...

    /// User-defined component for downstream DST.
    Custom(Cow<'static, str>),
}
```

Or use a string-based identifier instead of an enum entirely:

```rust
pub struct ComponentId(Cow<'static, str>);

impl ComponentId {
    pub const HEARTBEAT: Self = Self(Cow::Borrowed("durable::heartbeat"));
    pub const LEADER: Self = Self(Cow::Borrowed("durable::leader"));
    // etc.
}
```

---

## 7. Concrete Inventory of Changes

### Runtime-internal `Utc::now()` calls to replace with `clock.now()`

| File | Line | Context |
|------|------|---------|
| `worker.rs` | ~508 | `validate_workers`: compute next check time |
| `worker.rs` | ~598-600 | `leader`: compute delay to next wakeup |
| `plugin/wasi/io.rs` | ~231 | `pollable.ready`: check if pollable is expired |
| `plugin/wasi/io.rs` | ~268-269 | `pollable.block`: compute sleep delta |
| `plugin/wasi/io.rs` | ~363 | `poll`: check pollable expiry in loop |

### `tokio::time::sleep` / `sleep_until` calls to replace

| File | Line | Context |
|------|------|---------|
| `worker.rs` | ~356 | `heartbeat`: sleep between heartbeats |
| `worker.rs` | ~415 | `validate_workers`: sleep until next check |
| `worker.rs` | ~546-547 | `leader`: sleep until next wakeup scan |
| `plugin/wasi/io.rs` | ~288 | `pollable.block`: sleep for poll delta |
| `plugin/wasi/io.rs` | ~409 | `poll`: sleep in poll loop |

### `rand` calls to replace

| File | Line | Context |
|------|------|---------|
| `worker.rs` | ~379 | `heartbeat`: jitter calculation |

### Places to insert `scheduler.acquire()`

| Component | Location | Granularity |
|-----------|----------|-------------|
| `heartbeat` | Top of loop, after sleep/select | Per-heartbeat |
| `validate_workers` | Top of loop, after sleep/select | Per-validation-pass |
| `leader` | Top of loop, after sleep/select | Per-wakeup-scan |
| `task_cleanup` | Top of loop, after tick | Per-cleanup-pass |
| `stuck_notify` | Top of loop, after tick | Per-unstick-pass |
| `process_events` | After receiving event from select | Per-event |
| `spawn_new_tasks` | Before the claim query | Per-spawn-batch |
| `TaskState::enter_impl` | Before DB read/write | Per-transaction-enter |
| `TaskState::exit_impl` | Before DB write | Per-transaction-exit |

---

## 8. Design Decisions

### Why three traits instead of one?

`Scheduler`, `Clock`, and `Entropy` serve different purposes:
- **Scheduler**: Controls interleaving (which component runs next)
- **Clock**: Controls time progression (what "now" means)
- **Entropy**: Controls randomness (deterministic jitter/selection)

They're orthogonal. You might want a deterministic clock but real scheduling,
or controlled scheduling but real time (for integration tests). Keeping them
separate allows composition.

### Why not abstract the database?

Replacing `sqlx::PgPool` would require reimplementing PostgreSQL semantics
(transaction isolation, `SKIP LOCKED`, cascading deletes, `LISTEN/NOTIFY`).
This is a massive undertaking and would test a different system. The real
value of DST here is testing that durable's components interact correctly
**through** PostgreSQL under adversarial scheduling.

### How does this interact with the existing `EventSource` trait?

`EventSource` controls which events arrive. The `Scheduler` controls when the
runtime processes them. Together they provide full control: `EventSource`
determines the "what" and `Scheduler` determines the "when".

For DST, you'd typically use `PgEventSource` (real LISTEN/NOTIFY from the
real database) but gate its processing through the scheduler. This tests
the real event flow. Alternatively, a custom `EventSource` can inject
synthetic events for edge-case testing.

### Single-threaded vs multi-threaded runtime

The `Scheduler` approach works on both, but single-threaded
(`tokio::runtime::Builder::new_current_thread()`) is preferred for DST because:
- Tokio's single-threaded runtime is deterministic given deterministic
  futures (it uses a FIFO queue)
- No OS thread scheduling non-determinism
- `tokio::time::pause()` can freeze real time progression

Multi-threaded tests remain possible (the scheduler still controls
interleaving) but are harder to reproduce exactly.

---

## 9. Implementation Roadmap

### Phase 1: Core traits and wiring
- Define `Scheduler`, `Clock`, `Entropy` traits with default impls
- Add to `SharedState` and `WorkerBuilder`
- Wire `clock.now()` into the 5 `Utc::now()` call sites in worker/io code
- Wire `clock.sleep()` into the 5 sleep call sites
- Wire `entropy.random_range()` into the 1 rand call site
- Wire `scheduler.acquire()` into the 10 component iteration points
- Wire `scheduler.notify()` at state transition points

### Phase 2: DST implementations
- `DstScheduler` with seeded random selection
- `DstClock` with explicit time advancement
- `DstEntropy` with seeded RNG
- Seed-based test runner
- Basic test: single worker, single task, random scheduling
- Verify determinism: same seed → same event log

### Phase 3: Interesting tests
- Multi-worker with worker death and recovery
- Task suspend/wakeup under adversarial timing
- Leader election changes mid-operation
- Concurrent task spawning and claiming
- Fault injection (drop scheduler steps to simulate crashes)

### Phase 4: Downstream DST support
- Make `Scheduler`, `Clock`, `Entropy` public
- Add `Component::Custom` variant for user components
- Document DST patterns for systems built on durable
- Provide example DST test harness

---

## 10. References

- [FoundationDB: Testing Distributed Systems w/ Deterministic Simulation](https://apple.github.io/foundationdb/testing.html)
- [S2: Deterministic Simulation Testing for Async Rust](https://s2.dev/blog/dst)
- [Polar Signals: DST in Rust](https://www.polarsignals.com/blog/posts/2025/07/08/dst-rust)
- [TigerBeetle: Vortex + DST](https://tigerbeetle.com/blog/2025-02-13-a-descent-into-the-vortex/)
- [Antithesis: Deterministic Hypervisor](https://antithesis.com)
- [madsim](https://github.com/madsim-rs/madsim)
- [turmoil](https://github.com/tokio-rs/turmoil)
- [Resonate: DST for Durable Execution](https://blog.resonatehq.io/deterministic-simulation-testing)
