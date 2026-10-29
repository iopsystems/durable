# TLA+ model of the durable cluster

`Durable.tla` models the durable worker cluster: the worker table and
heartbeats, leader election, the pg_notify triggers and the `process_events`
loop, task claiming with `max_tasks`, and task executions that replay, commit,
sleep and wait for notifications, with or without a timeout. The aim is to find executions where the
cluster gets stuck or a task never completes, other than a workflow that
waits forever on its own.

Workflows are abstracted to a fixed list of durable transactions (`Prog`).
Every notification a plain wait is waiting for is eventually sent (a timed
wait's notification may never come), sleeps and deadlines eventually expire,
and programs never fail by themselves. So any failed task, or any task
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

| Config       | Workflow                    | Faults                                       |
|--------------|-----------------------------|----------------------------------------------|
| `crash.cfg`  | wait, then sleep            | crash, transient task errors, LISTEN drop    |
| `zombie.cfg` | wait, then sleep            | a worker misses heartbeats but keeps running, transient task errors, LISTEN drop |
| `timed.cfg`  | `notification_blocking_timeout` | same as `zombie.cfg`                     |
| `zombie-locked.cfg` | wait, then sleep     | same as `zombie.cfg`, plus the proposed fix for #133 (`CommitLocksTask`) |

All properties are expected to hold except in `zombie.cfg`, which finds
[#133](https://github.com/iopsystems/durable/issues/133).

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
- Each task has at most one sleep or timed wait, so its deadline is a single
  boolean. A timed wait records its deadline as a separate event first; the
  configs model that with a `"txn"` step before the `"twait"`.
- `commit_event_with_log` checks `running_on` against the statement's
  snapshot, so an execution that has just lost its task can still insert an
  event. The model lets a stale execution do that at any point after it loses
  the task, which is more often than it can really happen.
- Executions are a set, so two identical executions of a task on the same
  worker collapse into one.

## Findings

TLC (v1.7.4) results on a 4-core machine, for the code as of #130:

| Config       | Result    | Distinct states | Time   |
|--------------|-----------|-----------------|--------|
| `zombie.cfg` | `AllTasksComplete` violated, 19-step trace (#133) | 1,553,189 (partial) | 17min |
| `zombie-locked.cfg` | no errors                         | 2,339,527       | 34min  |
| `timed.cfg`  | no errors                                | 3,004,070       | 42min  |
| `crash.cfg`  | no errors; stopped before finishing (4.0M states still queued). Not re-run since #130 | 10,798,926 (partial) | 2h |

### Open: a stale `running_on` snapshot can still strand a task

Issue: [#133](https://github.com/iopsystems/durable/issues/133).

`commit_event_with_log` checks `running_on` in a plain `SELECT`, so the
check reads the statement's snapshot. #130 handles the event primary-key
conflict this can cause, but `zombie.cfg` finds a trace with no conflict
(worker 1 is the zombie, worker 2 the new owner, `Prog = <<"wait", "sleep">>`):

1. Worker 1's execution is in `notification_blocking`. In one transaction it
   deletes the notification and inserts event 0. The `running_on` check
   still sees worker 1.
2. Before that commits, worker 2 reaps worker 1 and claims the task. Its
   execution polls, waits on the notification row worker 1 has locked, and
   finds nothing once worker 1 commits.
3. Worker 2 hits `suspend_timeout`, and `try_suspend` suspends the task with
   no `wakeup_at`.

The task is `suspended`, with no timer and no pending notification, so
nothing wakes it again.

**Proposed fix** (`CommitLocksTask = TRUE`, `zombie-locked.cfg`): add
`FOR SHARE` to `current_task`. That conflicts with the reaper's
`running_on = NULL` update, so either the check sees `NULL`, or the reaper
waits until the event has committed and the new owner replays it.

### Fixed

The first version of the model found three bugs, all since fixed:

- [#125](https://github.com/iopsystems/durable/issues/125) and
  [#126](https://github.com/iopsystems/durable/issues/126), fixed in #130. If a
  worker misses its heartbeats but keeps running, another worker deletes its
  row and claims its tasks. The old worker's executions kept running, and
  several of their writes only filtered on the task id. TLC found two kinds of
  bad trace: the stale execution's `try_suspend` suspended the task after the
  new owner had already consumed its notification, leaving it suspended
  forever; or both executions recorded the same event, and the primary key
  conflict (or a suspend of an already-completed task) failed the task. Every
  write an execution makes to its own task is now conditional on
  `running_on = worker_id`, and an event conflict is retried instead of
  failing the task. The two original traces no longer reproduce, but see
  #133 above.
- [#127](https://github.com/iopsystems/durable/issues/127), fixed in #129. The
  leader computed its sleep as `now - wakeup_at` and so spun until the next
  suspended task was due. The model has no clock, so it cannot see this; it
  was found by reading the code.

Two other fixes also changed what the model does:

- #116: a timed notification wait that hits `suspend_timeout` now suspends
  with its deadline as `wakeup_at`. Before, it suspended with no timer and
  was stranded if its notification never came. Changing `ExecTimeout` back to
  leave the timer alone makes `timed.cfg` fail with that trace.
- #132: `Event::Lagged` now wakes every notification wait on the worker so it
  re-polls (`ProcessLagged`). Before, a wait whose notification event was lost
  sat until `suspend_timeout`. That is a delay, not a hang, so it did not
  break any property.
