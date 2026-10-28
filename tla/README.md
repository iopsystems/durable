# TLA+ model of the durable cluster

`Durable.tla` models the durable worker cluster: the worker table and
heartbeats, leader election, the pg_notify triggers and the `process_events`
loop, task claiming with `max_tasks`, and task executions that replay, commit,
sleep and wait for notifications. The aim is to find executions where the
cluster gets stuck or a task never completes, other than a workflow that
waits forever on its own.

Workflows are abstracted to a fixed list of durable transactions (`Prog`).
Every notification a workflow waits for is eventually sent, sleeps eventually
expire, and programs never fail by themselves. So any failed task, or any task
that never reaches `complete`, is a bug in the runtime.

## Running it

```sh
tla/check.sh                 # every config
tla/check.sh zombie.cfg      # just one
```

The script needs Java 11+ and downloads `tla2tools.jar` into `tla/.tools` on
first use (set `TLA2TOOLS_JAR` to use an existing copy). You can also open
`MC.tla` in the TLA+ Toolbox or the VS Code extension and point it at one of
the `.cfg` files.

| Config              | Faults                                      | Expected result                    |
|---------------------|---------------------------------------------|------------------------------------|
| `crash.cfg`         | crash, transient task errors, LISTEN drop   | all properties hold                |
| `zombie.cfg`        | a worker misses heartbeats but keeps running | `NoSpuriousFailure` violated      |
| `zombie-stuck.cfg`  | same, checks only `AllTasksFinish`          | `AllTasksFinish` violated (stuck)  |
| `zombie-fenced.cfg` | same, plus the proposed fix (`Fenced`)      | all properties hold                |

## Properties

- `NoSpuriousFailure`: no task ends up `failed`.
- `FinalIsFinal`: a `complete` or `failed` task never changes state again.
- `AllTasksComplete` / `AllTasksFinish`: every task eventually completes
  (or at least reaches a final state).
- `DeadRowsReaped`: the row of a dead worker is eventually deleted.
- `ExecsBehindLog`: sanity check that no execution is ahead of the event log.

## What the model abstracts

- Each database transaction is one atomic step. Row locks make this sound for
  the pairs that race in practice (e.g. `try_suspend` against a notification
  insert). Work between transactions interleaves freely.
- LISTEN events are kept as per-worker sets of pending wakeups, not a queue.
  The handlers re-read the database, so their order does not matter; dropping
  the connection loses them and delivers `Lagged`.
- The leader's background loops wake at least every 60s/600s, so they are
  modelled as always eventually running while they have work.
- `started_at` is assumed to increase with worker id.
- Each task has at most one sleep, so its deadline is a single boolean.
- Executions are a set, so two identical executions of a task on the same
  worker collapse into one.

## Findings

TLC (v1.7.4) results on a 4-core machine:

| Config              | Result                                         | Distinct states | Time   |
|---------------------|------------------------------------------------|-----------------|--------|
| `zombie.cfg`        | `NoSpuriousFailure` violated, 19-step trace    | 86,730 (partial)| 28s    |
| `zombie-stuck.cfg`  | `AllTasksFinish` violated, 26-step trace       | 241,012         | 29s    |
| `zombie-fenced.cfg` | no errors                                      | 2,339,527       | 17min  |
| `crash.cfg`         | running at time of writing; no errors at 1.6M  |                 |        |

### A worker that misses its heartbeat can fail or wedge the task it was running

Issues: [#125](https://github.com/iopsystems/durable/issues/125) (stuck),
[#126](https://github.com/iopsystems/durable/issues/126) (wrongly failed).

If a worker cannot heartbeat for `heartbeat_timeout` (120s by default) but is
still running, another worker deletes its row. Possible causes are a long
pause, CPU starvation, or a saturated connection pool: `heartbeat` goes
through `SharedState::acquire`, which retries with backoff, while tasks that
already hold connections keep going. Deleting the row sets `running_on = NULL`
on its tasks and `task_released` hands them to another worker. The old worker
does not find out until its next heartbeat, and its task executions keep
running until then.

Several writes that an execution makes to its own task only filter on `id`:

- `mark_task_complete` and `mark_task_failed`
- `suspend_task` (wasi sleep) and `suspend_task_no_wakeup` (`try_suspend`)
- the `durable.event` insert in `commit_event_with_log`. The comment in
  `exit_impl` says every statement there depends on `running_on = worker_id`,
  but the insert does not.

That leads to two kinds of bad trace (worker 1 is the zombie, worker 2 the new
owner, `Prog = <<"wait", "sleep">>`):

**Stuck forever** (`zombie-stuck.cfg`):

1. Worker 1 runs the task, which waits in `notification_blocking`.
2. Worker 1 misses its heartbeat. Worker 2 deletes its row and claims the task.
3. A notification arrives. Worker 2's execution consumes it and records event 0.
4. Worker 1's execution reaches `suspend_timeout` and calls `try_suspend`. The
   row is `active` and there are no notifications left, so it sets the task to
   `suspended` with no `wakeup_at`.
5. Worker 2's next commit sees `running_on = NULL` and stops with
   `NotScheduledOnWorker`.

The task is now `suspended`, with no timer and no pending notification.
`unwedge_stuck_notifications` only looks at tasks with a pending notification,
so nothing ever wakes it.

**Wrongly failed** (`zombie.cfg`): both executions write to the same task, so
either of these happens:

- One execution records event *i* between the other's `enter` and `exit`. The
  second `INSERT` hits the `durable.event` primary key, the error is not
  recoverable, and `mark_task_failed` fails the task. This is the trace TLC
  reports: the zombie inserts event 1 and then stops with
  `NotScheduledOnWorker`, and the new owner hits the conflict and fails the task.
- Worker 2 completes the task (`wasm = NULL`). Worker 1 then tries to suspend
  it, `check_wasm_while_active` rejects the update, and that error also ends
  in `mark_task_failed`. The task goes from `complete` to `failed`, which also
  violates `FinalIsFinal`.

**Proposed fix** (`Fenced = TRUE`): make every write an execution makes to its
own task conditional on `running_on = $worker_id`, and treat "no row updated"
as `NotScheduledOnWorker`:

- add `AND running_on = $2` to `mark_task_complete`, `mark_task_failed`,
  `suspend_task` and `suspend_task_no_wakeup`;
- in `try_suspend`, check after taking the row lock that the task is still
  `active` on this worker;
- make the `insert_event` CTE select from `current_task` only when
  `running_on` matches, and treat a primary-key conflict as a lost race (reset
  for retry) instead of a workflow error.

A zombie cannot re-claim its own tasks, because `fk_worker` rejects
`running_on` values with no worker row. The model includes that.

### Not modelled: the leader loop spins while a timer is pending

Issue: [#127](https://github.com/iopsystems/durable/issues/127).

In `Worker::leader` (`worker.rs`), the next sleep is computed as

```rust
now.signed_duration_since(wakeup_at)   // now - wakeup_at
```

When the next `wakeup_at` is in the future this is negative, `to_std()` fails,
and the leader sleeps for `Duration::ZERO`. It then reruns
`wake_suspended_tasks` and `next_wakeup_at` in a tight loop until the timer is
due. Tasks still wake on time, so the model, which has no clock, cannot see
this. It should be `wakeup_at + suspend_margin - now`.
