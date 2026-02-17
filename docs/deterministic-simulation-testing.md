# Deterministic Simulation Testing (DST)

This guide explains how to use the deterministic simulation testing infrastructure in `durable-runtime` to write reproducible tests of worker behavior.

## Overview

DST gives you deterministic control over three aspects of the runtime that are normally non-deterministic:

- **Scheduler** — which worker component runs next and when
- **Clock** — what time the runtime sees, and when sleeps complete
- **Entropy** — random values (e.g. heartbeat jitter)

A fourth primitive, **DstEventSource**, lets you control when PostgreSQL LISTEN/NOTIFY events are delivered to the worker.

PostgreSQL is **not mocked**. DST controls the timing of interactions with a real database, which is where the interesting bugs live.

## Key Types

All types live in `durable_runtime::dst` and `durable_runtime::scheduler`.

### Traits (production vs. DST)

| Trait | Production default | DST implementation |
|---|---|---|
| `Scheduler` | `NoopScheduler` — never blocks | `DstScheduler` — records all events and acquire calls |
| `Clock` | `SystemClock` — real wall clock | `DstClock` — frozen time, manually advanced |
| `Entropy` | `SystemEntropy` — OS random | `DstEntropy` — seeded RNG, deterministic |
| `EventSource` | `PgEventSource` — real Postgres notifications | `DstEventSource` — test-injected events |

### Component

The `Component` enum identifies what is requesting permission to run:

```rust
Component::Heartbeat { worker_id }       // heartbeat timestamp update
Component::ValidateWorkers { worker_id }  // dead worker cleanup
Component::Leader { worker_id }           // wake suspended tasks
Component::TaskCleanup { worker_id }      // delete old completed tasks
Component::StuckNotify { worker_id }      // unwedge stuck tasks
Component::ProcessEvents { worker_id }    // handle incoming events
Component::SpawnTasks { worker_id }       // claim and spawn tasks
Component::TaskTransaction { task_id, label } // task entering/exiting a DB transaction
Component::Custom(name)                   // user-defined, for downstream systems
```

### ScheduleEvent

Events emitted by the runtime at state transitions, delivered via `Scheduler::notify`:

```rust
ScheduleEvent::WorkerRegistered { worker_id }
ScheduleEvent::WorkerDeleted { worker_id }
ScheduleEvent::LeaderChanged { new_leader }
ScheduleEvent::TaskClaimed { worker_id, task_id }
ScheduleEvent::TaskCompleted { task_id, success }
ScheduleEvent::TaskSuspended { task_id, wakeup_at }
ScheduleEvent::TransactionRecorded { task_id, index, label }
ScheduleEvent::TasksWoken { count }
```

## Writing a DST Test

DST tests are async tests that use `#[sqlx::test]` to get a real PostgreSQL connection pool. The basic pattern is:

1. Create DST components
2. Spawn a worker with those components injected
3. Drive the simulation (advance clock, inject events, wait for scheduling points)
4. Assert on recorded events and acquire calls
5. Shut down

### Minimal Example

```rust
use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use durable_runtime::dst::{DstClock, DstEntropy, DstScheduler};
use durable_runtime::scheduler::{Component, ScheduleEvent};
use durable_runtime::Config;

#[sqlx::test]
async fn worker_starts_and_stops(pool: sqlx::PgPool) -> anyhow::Result<()> {
    // 1. Create DST components
    let scheduler = Arc::new(DstScheduler::new(42));
    let clock = Arc::new(DstClock::new(Utc::now()));
    let entropy = Arc::new(DstEntropy::new(42));

    let config = Config::new()
        .suspend_margin(Duration::from_secs(1))
        .suspend_timeout(Duration::from_secs(1));

    // 2. Spawn a worker
    let guard = durable_test::spawn_worker_with_dst(
        pool,
        config,
        scheduler.clone(),
        clock.clone(),
        entropy.clone(),
    )
    .await?;

    // 3. Wait for scheduling activity
    tokio::time::timeout(Duration::from_secs(5), async {
        scheduler.wait_for_acquires(3).await;
    })
    .await?;

    // 4. Assert
    let events = scheduler.events();
    assert!(events.iter().any(|e| matches!(e, ScheduleEvent::WorkerRegistered { .. })));

    // 5. Shut down
    guard.handle().shutdown();
    scheduler
        .wait_for_event(|e| matches!(e, ScheduleEvent::WorkerDeleted { .. }))
        .await;

    Ok(())
}
```

### Spawning Workers

The `durable_test` crate provides two helpers:

```rust
// Without custom event source (uses default Postgres listener):
durable_test::spawn_worker_with_dst(pool, config, scheduler, clock, entropy).await?;

// With custom event source for injecting events:
durable_test::spawn_worker_with_dst_events(pool, config, scheduler, clock, entropy, event_source).await?;
```

Both return a `WorkerShutdownGuard`. Call `guard.handle().shutdown()` to initiate graceful shutdown. The guard also shuts down the worker on drop.

## DstScheduler

`DstScheduler` records all `acquire` and `notify` calls without blocking. It does not control execution order — it acts like `NoopScheduler` but with full observability.

### API

```rust
let scheduler = DstScheduler::new(seed);

// Observation
scheduler.events()           // -> Vec<ScheduleEvent> snapshot
scheduler.acquires()         // -> Vec<Component> snapshot
scheduler.acquire_count()    // -> u64

// Waiting
scheduler.wait_for_acquires(n).await;     // blocks until n acquire calls recorded
scheduler.wait_for_event(|e| ...).await;  // blocks until matching event recorded

// Reset
scheduler.clear();
```

Always wrap `wait_for_*` calls in `tokio::time::timeout` to avoid hanging on test failure:

```rust
tokio::time::timeout(Duration::from_secs(5), async {
    scheduler.wait_for_event(|e| matches!(e, ScheduleEvent::WorkerRegistered { .. })).await;
})
.await
.expect("worker should register within 5 seconds");
```

## DstClock

Time is frozen at construction and only moves when you advance it. Sleeps in the worker complete when the clock passes their deadline.

```rust
let clock = DstClock::new(Utc::now());

clock.advance(Duration::from_secs(10));  // advance by duration, wakes expired sleepers
clock.set(specific_time);                // jump to absolute time
clock.current_time()                     // read current simulated time
```

Components that sleep (heartbeat, validate_workers, leader, etc.) will remain blocked until you advance the clock past their sleep deadline. This is how you drive the simulation forward:

```rust
// Trigger heartbeats by advancing past the heartbeat interval (default 5s)
clock.advance(Duration::from_secs(10));
tokio::task::yield_now().await;  // let the runtime process the wakeups
```

## DstEntropy

Produces a deterministic sequence of random values from a seed. Same seed always gives the same sequence.

```rust
let entropy = DstEntropy::new(42);
entropy.seed()  // -> 42
```

The runtime uses entropy for heartbeat jitter. With a fixed seed, heartbeat timing is reproducible.

## DstEventSource

Controls delivery of PostgreSQL notification events to the worker. Split into two halves:

- `DstEventSource` — implements `EventSource`, given to the worker
- `DstEventHandle` — retained by the test, used to inject events

```rust
let (event_source, event_handle) = DstEventSource::new();

// Pass event_source to the worker
let guard = durable_test::spawn_worker_with_dst_events(
    pool, config, scheduler, clock, entropy,
    Box::new(event_source),
).await?;

// Inject events from the test
event_handle.send_task(task_id, running_on);
event_handle.send_worker(worker_id);
event_handle.send_notification(task_id, event_name);
event_handle.send_task_suspend(task_id);
event_handle.send_lagged();          // simulate connection loss
event_handle.is_closed()             // check if worker dropped its receiver
```

When no events are sent, the worker's event loop blocks waiting — this lets you control exactly when the worker sees new work.

## Common Patterns

### Advancing the Clock in a Loop

Many components sleep between iterations. To trigger multiple cycles, advance the clock in a loop:

```rust
tokio::time::timeout(Duration::from_secs(5), async {
    loop {
        let count = scheduler.acquires().iter()
            .filter(|c| matches!(c, Component::ValidateWorkers { .. }))
            .count();
        if count >= 2 { break; }
        clock.advance(Duration::from_secs(30));
        tokio::task::yield_now().await;
    }
}).await?;
```

### Multi-Worker Tests

Multiple workers can share the same `DstScheduler` and `DstClock`. Events from all workers are recorded in the shared scheduler, making it straightforward to test leader election and task distribution:

```rust
let scheduler = Arc::new(DstScheduler::new(42));
let clock = Arc::new(DstClock::new(Utc::now()));

let guard1 = durable_test::spawn_worker_with_dst(
    pool.clone(), config.clone(),
    scheduler.clone(), clock.clone(), Arc::new(DstEntropy::new(42)),
).await?;

let guard2 = durable_test::spawn_worker_with_dst(
    pool.clone(), config,
    scheduler.clone(), clock.clone(), Arc::new(DstEntropy::new(43)),
).await?;

// Wait for both workers to register
loop {
    let registered = scheduler.events().iter()
        .filter(|e| matches!(e, ScheduleEvent::WorkerRegistered { .. }))
        .count();
    if registered >= 2 { break; }
    tokio::task::yield_now().await;
}
```

Use different entropy seeds for each worker to avoid lock contention on the shared RNG mutex.

### Verifying Shutdown Cleanup

After calling `guard.handle().shutdown()`, advance the clock so sleeping components wake up and observe the shutdown signal:

```rust
guard1.handle().shutdown();
guard2.handle().shutdown();

tokio::time::timeout(Duration::from_secs(5), async {
    loop {
        let deleted = scheduler.events().iter()
            .filter(|e| matches!(e, ScheduleEvent::WorkerDeleted { .. }))
            .count();
        if deleted >= 2 { break; }
        clock.advance(Duration::from_secs(5));
        tokio::task::yield_now().await;
    }
}).await?;
```

## File Layout

```
crates/durable-runtime/src/
├── scheduler.rs        # Scheduler trait, Component, ScheduleEvent, ScheduleGuard
├── clock.rs            # Clock trait, SystemClock
├── entropy.rs          # Entropy trait, SystemEntropy
├── dst.rs              # DstScheduler, DstClock, DstEntropy, DstEventSource/Handle
├── event/mod.rs        # Event enum, EventSource trait
└── worker.rs           # WorkerBuilder (accepts scheduler/clock/entropy/event_source)

crates/durable-test/
├── src/lib.rs          # spawn_worker_with_dst, spawn_worker_with_dst_events
└── tests/dst.rs        # Integration tests (reference examples)
```

## Running DST Tests

DST tests require a PostgreSQL database. They use `#[sqlx::test]` which automatically creates a temporary database per test.

```bash
cargo test -p durable-test --test dst
```

To run a specific test:

```bash
cargo test -p durable-test --test dst worker_lifecycle_with_dst_hooks
```
