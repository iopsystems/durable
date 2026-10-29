------------------------------- MODULE Durable -------------------------------
(***************************************************************************)
(* A model of the durable worker cluster, written to find executions in    *)
(* which the cluster gets stuck or a task never reaches a final state.     *)
(*                                                                         *)
(* What is modelled, and where it lives in the code:                       *)
(*                                                                         *)
(*  - The `durable.worker` table, heartbeats and worker reaping            *)
(*    (worker.rs: heartbeat, validate_workers).                            *)
(*  - The leader election cache and the leader-only loops that wake timed  *)
(*    out tasks and unwedge tasks with stuck notifications (worker.rs:     *)
(*    load_leader_id, leader, stuck_notify).                               *)
(*  - The `durable.task` row (state, running_on, wakeup_at), the           *)
(*    `durable.event` log and the `durable.notification` queue.            *)
(*  - The pg_notify triggers in migrations/*.sql, delivered to each        *)
(*    worker's LISTEN connection, and the process_events loop that reacts  *)
(*    to them (claim_tasks, release_owned_ready_tasks, max_tasks).         *)
(*  - Task executions: replaying recorded events, committing new ones      *)
(*    (commit_event_with_log), sleeping (wasi poll -> suspend_task),       *)
(*    waiting for notifications (notify.rs: notification_blocking,         *)
(*    notification_blocking_timeout, try_suspend) and finishing            *)
(*    (mark_task_complete / mark_task_failed). Every write an execution    *)
(*    makes to its own task is fenced on running_on = this worker (#130).  *)
(*  - Faults: worker crashes, a worker that misses heartbeats while still  *)
(*    running ("zombie"), transient database errors inside a task, and     *)
(*    dropped LISTEN connections.                                          *)
(*                                                                         *)
(* Granularity: every database transaction is one atomic step. Postgres    *)
(* row locks make that sound for the pairs of transactions that matter     *)
(* here (see notify.rs try_suspend for the notification/suspend pair).     *)
(* Work done in memory between two transactions is where interleavings     *)
(* happen, and those are separate steps.                                   *)
(*                                                                         *)
(* Workflows are abstracted to a fixed sequence of steps, `Prog`. Each     *)
(* step is one durable transaction:                                        *)
(*   "txn"   - any transaction that just records a result (e.g. http).    *)
(*   "wait"  - notification_blocking: consumes one notification.           *)
(*   "sleep" - a long sleep that suspends the task until a timer fires.    *)
(*   "twait" - notification_blocking_timeout: returns a notification if    *)
(*             one arrives before the deadline, and None otherwise. Its    *)
(*             deadline is recorded by a separate event first; write a     *)
(*             "txn" step before it to model that.                         *)
(* Programs never fail on their own, never wait for a notification that is *)
(* not eventually sent and never block forever. So any task that fails or  *)
(* never completes is the runtime's fault, not the workflow's.             *)
(***************************************************************************)
EXTENDS Naturals, FiniteSets, Sequences, TLC

CONSTANTS
    Workers,        \* Worker ids (bigserial) in the order workers register.
    Tasks,          \* Task ids.
    Prog,           \* The workflow every task runs, a sequence of steps.
    MaxTasks,       \* Config::max_tasks.
    CrashWorkers,   \* Workers whose process may die, leaving their row behind.
    FlakyWorkers,   \* Workers that may miss a heartbeat while still running.
    MaxLate,        \* Bound on missed heartbeats, across all workers.
    MaxTransient,   \* Bound on transient (Io/PoolTimedOut) errors in tasks.
    MaxReconnects,  \* Bound on dropped LISTEN connections.
    CommitLocksTask,\* TRUE models the proposed FOR SHARE in commit_event_with_log.
    Null            \* SQL NULL.

ASSUME
    /\ IsFiniteSet(Workers) /\ Workers \subseteq Nat /\ Workers # {}
    /\ IsFiniteSet(Tasks) /\ Tasks \subseteq Nat /\ Tasks # {}
    /\ Null \notin Workers
    /\ \A i \in 1..Len(Prog) : Prog[i] \in {"txn", "wait", "sleep", "twait"}
    \* `deadline` below tracks a single timer per task.
    /\ Cardinality({i \in 1..Len(Prog) : Prog[i] \in {"sleep", "twait"}}) <= 1
    /\ MaxTasks \in Nat \ {0}
    /\ CrashWorkers \subseteq Workers
    /\ FlakyWorkers \subseteq Workers
    /\ {MaxLate, MaxTransient, MaxReconnects} \subseteq Nat
    /\ CommitLocksTask \in BOOLEAN

\* Every plain wait is eventually sent its notification. A timed wait may
\* or may not be; that is what the timeout is for.
NumWaits == Cardinality({i \in 1..Len(Prog) : Prog[i] = "wait"})
NumSends == Cardinality({i \in 1..Len(Prog) : Prog[i] \in {"wait", "twait"}})

Min(S) == CHOOSE x \in S : \A y \in S : x <= y
Max(S) == CHOOSE x \in S : \A y \in S : y <= x

TaskStates == {"none", "ready", "active", "suspended", "complete", "failed"}
Final      == {"complete", "failed"}
Phases     == {"run", "commit", "sleep", "poll", "waiting", "tpoll",
               "twaiting", "error"}

VARIABLES
    \* ---- worker processes and the durable.worker table ----
    wstatus,    \* Worker process: "unborn", "running" or "dead".
    rows,       \* Ids that have a row in durable.worker.
    expired,    \* Rows whose heartbeat_at is older than heartbeat_timeout.
    following,  \* validate_workers: the worker this one is watching.
    leader,     \* SharedState::leader, this worker's idea of the leader.
    blocked,    \* Worker::blocked.

    \* ---- per-worker pending wakeups for the process_events loop ----
    evSpawn,    \* A `durable:task` event with running_on in {NULL, self}.
    evNotif,    \* Task ids from pending `durable:notification` events.
    evWorker,   \* Worker ids from pending `durable:worker` events.
    evLagged,   \* The LISTEN connection was re-established.
    joinWake,   \* A task in the JoinSet finished.

    \* ---- database state for tasks ----
    task,       \* durable.task: [state, ro (running_on), timer (wakeup_at)].
    nq,         \* Number of rows in durable.notification for each task.
    sent,       \* Number of notifications the outside world has sent.
    recorded,   \* Number of rows in durable.event for each task.
    deadline,   \* Whether the task's sleep or timed wait deadline passed.

    \* ---- task executions running inside worker processes ----
    execs,      \* Set of [w, t, pc, ph]; pc = next transaction index.

    \* ---- fault budgets ----
    late, transient, reconnects

workerVars == <<wstatus, rows, expired, following, leader, blocked>>
eventVars  == <<evSpawn, evNotif, evWorker, evLagged, joinWake>>
taskVars   == <<task, nq, sent, recorded, deadline>>
faultVars  == <<late, transient, reconnects>>
vars       == <<workerVars, eventVars, taskVars, execs, faultVars>>

Exec == [w : Workers, t : Tasks, pc : 0..Len(Prog), ph : Phases]

TypeOK ==
    /\ wstatus \in [Workers -> {"unborn", "running", "dead"}]
    /\ rows \subseteq Workers
    /\ expired \subseteq Workers
    /\ following \in [Workers -> Workers \cup {Null}]
    /\ leader \in [Workers -> Workers \cup {Null}]
    /\ blocked \in [Workers -> BOOLEAN]
    /\ evSpawn \in [Workers -> BOOLEAN]
    /\ evNotif \in [Workers -> SUBSET Tasks]
    /\ evWorker \in [Workers -> SUBSET Workers]
    /\ evLagged \in [Workers -> BOOLEAN]
    /\ joinWake \in [Workers -> BOOLEAN]
    /\ task \in [Tasks -> [state : TaskStates,
                           ro : Workers \cup {Null},
                           timer : BOOLEAN]]
    /\ nq \in [Tasks -> 0..NumSends]
    /\ sent \in [Tasks -> 0..NumSends]
    /\ recorded \in [Tasks -> 0..Len(Prog)]
    /\ deadline \in [Tasks -> BOOLEAN]
    /\ execs \subseteq Exec
    /\ late \in 0..MaxLate
    /\ transient \in 0..MaxTransient
    /\ reconnects \in 0..MaxReconnects

-----------------------------------------------------------------------------
(* Helpers *)

\* Workers whose LISTEN connection is up. A zombie (running, but its row was
\* deleted) still listens.
Listening == {w \in Workers : wstatus[w] = "running"}

\* load_leader_id: ORDER BY started_at, id LIMIT 1. We assume started_at
\* increases with id.
LeaderOf(S) == IF S = {} THEN Null ELSE Min(S)

\* Who acts on a `durable:task` event whose payload has running_on = r.
Relevant(r) == IF r = Null THEN Listening ELSE Listening \cap {r}

SpawnTo(S) == [x \in Workers |-> evSpawn[x] \/ x \in S]

\* The `running_on` a trigger or client picks with
\* `SELECT id FROM durable.worker ... FOR SHARE SKIP LOCKED LIMIT 1`.
\* Rows locked by a concurrent heartbeat are skipped, so NULL is possible.
PickSkipLocked == rows \cup {Null}

Owns(e) == task[e.t].ro = e.w

\* The process exits. Its in-flight task futures are dropped, which rolls
\* back any open database transaction. Its row (if any) is left alone: a crash
\* leaves it for validate_workers, and heartbeat-detected shutdown only
\* happens once the row is already gone.
Shutdown(w) ==
    /\ wstatus' = [wstatus EXCEPT ![w] = "dead"]
    /\ execs' = {e \in execs : e.w # w}

-----------------------------------------------------------------------------
(* Worker::spawn_new_tasks. One database transaction: claim_tasks, then
   release_owned_ready_tasks if this worker is now full.

   `clear` says whether this call consumes the worker's pending evSpawn.
   `base` is the worker's set of executions before the claimed ones are
   added. Sets task, execs, blocked, evSpawn and wstatus. *)
Spawn(w, clear, base) ==
    LET mine      == {e \in execs : e.w = w}
        len       == Cardinality(mine)
        claimable == {t \in Tasks :
                        \/ /\ task[t].state \in {"ready", "active"}
                           /\ task[t].ro = Null
                        \/ /\ task[t].state = "ready"
                           /\ task[t].ro = w}
        allowed   == MaxTasks - len
        \* ORDER BY id LIMIT allowed
        claimed   == {t \in claimable :
                        Cardinality({u \in claimable : u < t}) < allowed}
        full      == Cardinality(claimed) + len >= MaxTasks
        released  == IF full
                     THEN {t \in Tasks \ claimed :
                             task[t].state = "ready" /\ task[t].ro = w}
                     ELSE {}
        selfEv    == IF clear THEN FALSE ELSE evSpawn[w]
    IN  IF len >= MaxTasks
        THEN \* `allowed == 0`: return before doing anything.
             /\ evSpawn' = [evSpawn EXCEPT ![w] = selfEv]
             /\ UNCHANGED <<task, execs, blocked, wstatus>>
        ELSE IF w \notin rows /\ claimed # {}
        THEN \* A zombie tries to set running_on to its own, deleted, id.
             \* fk_worker rejects the UPDATE, process_events returns the
             \* error and the worker shuts down.
             /\ evSpawn' = [evSpawn EXCEPT ![w] = selfEv]
             /\ Shutdown(w)
             /\ UNCHANGED <<task, blocked>>
        ELSE
               /\ task' = [t \in Tasks |->
                             IF t \in claimed
                             THEN [task[t] EXCEPT !.state = "active", !.ro = w]
                             ELSE IF t \in released
                                  THEN [task[t] EXCEPT !.ro = Null]
                                  ELSE task[t]]
               /\ execs' = base \cup
                             {[w |-> w, t |-> t, pc |-> 0, ph |-> "run"] :
                                t \in claimed}
               /\ blocked' = IF full THEN [blocked EXCEPT ![w] = TRUE]
                                     ELSE blocked
               \* task_released fires for every released row and reaches
               \* every listening worker, including this one.
               /\ evSpawn' = [x \in Workers |->
                                IF x = w
                                THEN selfEv \/ released # {}
                                ELSE evSpawn[x] \/
                                       (released # {} /\ x \in Listening)]
               /\ UNCHANGED wstatus

-----------------------------------------------------------------------------
(* Worker lifecycle *)

\* Worker::run: insert_worker, load_leader_id, then process_events starts with
\* spawn_new_tasks. Ids are handed out in order.
Start(w) ==
    /\ wstatus[w] = "unborn"
    /\ \A v \in Workers : v < w => wstatus[v] # "unborn"
    /\ wstatus' = [wstatus EXCEPT ![w] = "running"]
    /\ rows' = rows \cup {w}
    /\ leader' = [leader EXCEPT ![w] = LeaderOf(rows \cup {w})]
    /\ evSpawn' = [evSpawn EXCEPT ![w] = TRUE]
    /\ evWorker' = [x \in Workers |->
                      IF x \in Listening THEN evWorker[x] \cup {w}
                                         ELSE evWorker[x]]
    /\ UNCHANGED <<expired, following, blocked, evNotif, evLagged, joinWake,
                   taskVars, execs, faultVars>>

Crash(w) ==
    /\ w \in CrashWorkers
    /\ wstatus[w] = "running"
    /\ Shutdown(w)
    /\ UNCHANGED <<rows, expired, following, leader, blocked, eventVars,
                   taskVars, faultVars>>

\* A dead worker stops heartbeating, so its row eventually expires.
Expire(w) ==
    /\ wstatus[w] = "dead"
    /\ w \in rows
    /\ w \notin expired
    /\ expired' = expired \cup {w}
    /\ UNCHANGED <<wstatus, rows, following, leader, blocked, eventVars,
                   taskVars, execs, faultVars>>

\* A live worker is too slow to heartbeat (GC pause, overloaded runtime,
\* network partition) and its row looks expired.
LateHeartbeat(w) ==
    /\ w \in FlakyWorkers
    /\ wstatus[w] = "running"
    /\ w \in rows
    /\ w \notin expired
    /\ late < MaxLate
    /\ late' = late + 1
    /\ expired' = expired \cup {w}
    /\ UNCHANGED <<wstatus, rows, following, leader, blocked, eventVars,
                   taskVars, execs, transient, reconnects>>

\* Worker::heartbeat while the row still exists.
HeartbeatRefresh(w) ==
    /\ wstatus[w] = "running"
    /\ w \in rows
    /\ w \in expired
    /\ expired' = expired \ {w}
    /\ UNCHANGED <<wstatus, rows, following, leader, blocked, eventVars,
                   taskVars, execs, faultVars>>

\* Worker::heartbeat finds its row gone and shuts the worker down.
HeartbeatDetect(w) ==
    /\ wstatus[w] = "running"
    /\ w \notin rows
    /\ Shutdown(w)
    /\ UNCHANGED <<rows, expired, following, leader, blocked, eventVars,
                   taskVars, faultVars>>

\* Worker::validate_workers, one iteration (a single transaction).
\* Deleting a row sets running_on = NULL on its tasks (fk_worker ON DELETE
\* SET NULL), which fires task_released for those still ready/active, and
\* fires worker_deleted.
Validate(w) ==
    LET f        == following[w]
        del      == IF f # Null /\ f \in rows \cap expired
                    THEN {f} \cup ((rows \cap expired) \ {w})
                    ELSE {}
        rows2    == rows \ del
        lower    == {v \in rows2 : v < w}
        others   == rows2 \ {w}
        nxt      == IF lower # {} THEN Max(lower)
                    ELSE IF others # {} THEN Max(others) ELSE Null
        orphaned == {t \in Tasks : task[t].ro \in del}
        released == {t \in orphaned : task[t].state \in {"ready", "active"}}
    IN  /\ wstatus[w] = "running"
        /\ rows' = rows2
        /\ expired' = expired \ del
        /\ following' = [following EXCEPT ![w] = nxt]
        /\ task' = [t \in Tasks |->
                      IF t \in orphaned THEN [task[t] EXCEPT !.ro = Null]
                                        ELSE task[t]]
        /\ evSpawn' = SpawnTo(IF released # {} THEN Listening ELSE {})
        /\ evWorker' = [x \in Workers |->
                          IF x \in Listening THEN evWorker[x] \cup del
                                             ELSE evWorker[x]]
        /\ UNCHANGED <<wstatus, leader, blocked, evNotif, evLagged, joinWake,
                       nq, sent, recorded, deadline, execs, faultVars>>

-----------------------------------------------------------------------------
(* Leader-only background loops. They wake at least every 60s/600s, so we
   treat them as always eventually running while there is work for them. *)

\* Worker::leader -> wake_suspended_tasks. Each woken task gets a random
\* worker row (no SKIP LOCKED here, so never NULL while rows exist).
LeaderWake(w) ==
    LET due     == {t \in Tasks : /\ task[t].state = "suspended"
                                  /\ task[t].timer
                                  /\ deadline[t]}
        targets == IF rows = {} THEN {Null} ELSE rows
    IN  /\ wstatus[w] = "running"
        /\ leader[w] = w
        /\ due # {}
        /\ \E assign \in [due -> targets] :
              /\ task' = [t \in Tasks |->
                            IF t \in due
                            THEN [state |-> "ready", ro |-> assign[t],
                                  timer |-> FALSE]
                            ELSE task[t]]
              \* task_updated: suspended -> ready.
              /\ evSpawn' = [x \in Workers |->
                               \/ evSpawn[x]
                               \/ /\ x \in Listening
                                  /\ \E t \in due : assign[t] \in {Null, x}]
        /\ UNCHANGED <<workerVars, evNotif, evWorker, evLagged, joinWake,
                       nq, sent, recorded, deadline, execs, faultVars>>

\* Worker::stuck_notify -> unwedge_stuck_notifications. Only changes state;
\* running_on and wakeup_at are left as they were.
Unwedge(w) ==
    LET stuck == {t \in Tasks : task[t].state = "suspended" /\ nq[t] > 0}
    IN  /\ wstatus[w] = "running"
        /\ leader[w] = w
        /\ stuck # {}
        /\ task' = [t \in Tasks |->
                      IF t \in stuck THEN [task[t] EXCEPT !.state = "ready"]
                                     ELSE task[t]]
        /\ evSpawn' = [x \in Workers |->
                         \/ evSpawn[x]
                         \/ \E t \in stuck : x \in Relevant(task[t].ro)]
        /\ UNCHANGED <<workerVars, evNotif, evWorker, evLagged, joinWake,
                       nq, sent, recorded, deadline, execs, faultVars>>

-----------------------------------------------------------------------------
(* Worker::process_events. Each loop iteration handles one wakeup. *)

PEUnchanged == UNCHANGED <<rows, expired, following, nq, sent, recorded,
                           deadline, faultVars>>

\* LoopEvent::TaskComplete
ProcessJoin(w) ==
    /\ wstatus[w] = "running"
    /\ joinWake[w]
    /\ joinWake' = [joinWake EXCEPT ![w] = FALSE]
    /\ IF blocked[w] THEN Spawn(w, FALSE, execs)
                     ELSE UNCHANGED <<task, execs, blocked, evSpawn, wstatus>>
    /\ UNCHANGED <<leader, evNotif, evWorker, evLagged>>
    /\ PEUnchanged

\* Event::Task with running_on in {NULL, self}.
ProcessSpawn(w) ==
    /\ wstatus[w] = "running"
    /\ evSpawn[w]
    /\ Spawn(w, TRUE, execs)
    /\ UNCHANGED <<leader, evNotif, evWorker, evLagged, joinWake>>
    /\ PEUnchanged

\* Executions on worker w, of a task in ts, that are waiting for a
\* notification go back to polling.
RepollSet(w, ts) ==
    {IF e.w = w /\ e.t \in ts /\ e.ph \in {"waiting", "twaiting"}
              THEN [e EXCEPT !.ph = IF e.ph = "waiting" THEN "poll" ELSE "tpoll"]
              ELSE e : e \in execs}

Repoll(w, ts) == execs' = RepollSet(w, ts)

\* Event::Notification: rebroadcast to this worker's tasks. Executions of
\* that task waiting for a notification go back to polling.
ProcessNotif(w, t) ==
    /\ wstatus[w] = "running"
    /\ t \in evNotif[w]
    /\ evNotif' = [evNotif EXCEPT ![w] = @ \ {t}]
    /\ Repoll(w, {t})
    /\ UNCHANGED <<wstatus, leader, blocked, evSpawn, evWorker, evLagged,
                   joinWake, task>>
    /\ PEUnchanged

\* Event::Worker: reload the leader if the cached one is the subject of the
\* event, or there is none.
ProcessWorker(w, v) ==
    /\ wstatus[w] = "running"
    /\ v \in evWorker[w]
    /\ evWorker' = [evWorker EXCEPT ![w] = @ \ {v}]
    /\ leader' = IF leader[w] \in {v, Null}
                 THEN [leader EXCEPT ![w] = LeaderOf(rows)]
                 ELSE leader
    /\ UNCHANGED <<wstatus, blocked, evSpawn, evNotif, evLagged, joinWake,
                   task, execs>>
    /\ PEUnchanged

\* Event::Lagged: wake every notification wait on this worker
\* (notification_resync), then spawn_new_tasks and load_leader_id.
ProcessLagged(w) ==
    /\ wstatus[w] = "running"
    /\ evLagged[w]
    /\ evLagged' = [evLagged EXCEPT ![w] = FALSE]
    /\ Spawn(w, FALSE, RepollSet(w, Tasks))
    /\ leader' = [leader EXCEPT ![w] = LeaderOf(rows)]
    /\ UNCHANGED <<evNotif, evWorker, joinWake>>
    /\ PEUnchanged

\* The LISTEN connection drops. Anything sent meanwhile is lost; PgEventSource
\* reports Event::Lagged once it reconnects.
Reconnect(w) ==
    /\ wstatus[w] = "running"
    /\ reconnects < MaxReconnects
    /\ reconnects' = reconnects + 1
    /\ evSpawn' = [evSpawn EXCEPT ![w] = FALSE]
    /\ evNotif' = [evNotif EXCEPT ![w] = {}]
    /\ evWorker' = [evWorker EXCEPT ![w] = {}]
    /\ evLagged' = [evLagged EXCEPT ![w] = TRUE]
    /\ UNCHANGED <<workerVars, joinWake, taskVars, execs, late, transient>>

-----------------------------------------------------------------------------
(* Clients and the passage of time *)

\* durable-client launch: INSERT with a SKIP LOCKED random running_on.
\* Fires task_inserted.
Launch(t) ==
    /\ task[t].state = "none"
    /\ \E r \in PickSkipLocked :
          /\ task' = [task EXCEPT ![t] = [state |-> "ready", ro |-> r,
                                          timer |-> FALSE]]
          /\ evSpawn' = SpawnTo(Relevant(r))
    /\ UNCHANGED <<workerVars, evNotif, evWorker, evLagged, joinWake,
                   nq, sent, recorded, deadline, execs, faultVars>>

\* durable-client notify: INSERT INTO durable.notification. The
\* notify_notification trigger wakes the task if it is suspended. At most
\* `max` notifications are sent to each task.
SendNotify(t, max) ==
    /\ task[t].state # "none"
    /\ sent[t] < max
    /\ sent' = [sent EXCEPT ![t] = @ + 1]
    /\ nq' = [nq EXCEPT ![t] = @ + 1]
    /\ evNotif' = [x \in Workers |->
                     IF x \in Listening THEN evNotif[x] \cup {t}
                                        ELSE evNotif[x]]
    /\ IF task[t].state = "suspended"
       THEN \E r \in PickSkipLocked :
              /\ task' = [task EXCEPT ![t] = [state |-> "ready", ro |-> r,
                                              timer |-> FALSE]]
              /\ evSpawn' = SpawnTo(Relevant(r))
       ELSE UNCHANGED <<task, evSpawn>>
    /\ UNCHANGED <<workerVars, evWorker, evLagged, joinWake, recorded,
                   deadline, execs, faultVars>>

\* Time passes the task's sleep deadline (by at least suspend_margin).
Tick(t) ==
    /\ ~deadline[t]
    /\ deadline' = [deadline EXCEPT ![t] = TRUE]
    /\ UNCHANGED <<workerVars, eventVars, task, nq, sent, recorded, execs,
                   faultVars>>

-----------------------------------------------------------------------------
(* Task executions (Worker::run_task and the plugin host functions).

   An execution `e` runs the program from the start, replaying recorded
   events. Executions belonging to a zombie keep running until the zombie
   notices its row is gone. *)

ExecUnchanged == UNCHANGED <<workerVars, evNotif, evWorker, evLagged, sent,
                             deadline, faultVars>>

Advance(e, f) ==
    /\ execs' = (execs \ {e}) \cup {f}
    /\ UNCHANGED joinWake

\* The task future returns; the JoinSet wakes process_events.
Finish(e) ==
    /\ execs' = execs \ {e}
    /\ joinWake' = [joinWake EXCEPT ![e.w] = TRUE]

\* reset_task_for_retry: only if we still own it. Fires task_released.
ResetForRetry(e) ==
    IF Owns(e)
    THEN /\ task' = [task EXCEPT ![e.t].state = "ready", ![e.t].ro = Null]
         /\ evSpawn' = SpawnTo(Listening)
    ELSE UNCHANGED <<task, evSpawn>>

RecordEvent(t) == recorded' = [recorded EXCEPT ![t] = @ + 1]

\* TaskState::enter for the step at index pc: replay it if the event is
\* already recorded, otherwise start a new transaction.
ExecEnter(e) ==
    /\ e.ph = "run"
    /\ e.pc < Len(Prog)
    /\ IF e.pc < recorded[e.t]
       THEN Advance(e, [e EXCEPT !.pc = e.pc + 1])
       ELSE LET k   == Prog[e.pc + 1]
                nph == IF k = "wait" THEN "poll"
                       ELSE IF k = "twait" THEN "tpoll"
                       ELSE IF k = "sleep" /\ ~deadline[e.t] THEN "sleep"
                       ELSE "commit"
            IN  Advance(e, [e EXCEPT !.ph = nph])
    /\ UNCHANGED <<task, nq, recorded, evSpawn>>
    /\ ExecUnchanged

\* The workflow returned: TaskStatus::ExitSuccess -> mark_task_complete,
\* which only applies if we still own the task.
ExecComplete(e) ==
    /\ e.ph = "run"
    /\ e.pc = Len(Prog)
    /\ IF Owns(e)
       THEN task' = [task EXCEPT ![e.t].state = "complete", ![e.t].ro = Null]
       ELSE UNCHANGED task
    /\ Finish(e)
    /\ UNCHANGED <<nq, recorded, evSpawn>>
    /\ ExecUnchanged

\* Leave the execution: NotScheduledOnWorker, nothing written.
Quit(e) ==
    /\ Finish(e)
    /\ UNCHANGED <<task, evSpawn>>

\* Hit an error that is not recoverable: TaskStatus::ExitFailure.
Fail(e) ==
    /\ Advance(e, [e EXCEPT !.ph = "error"])
    /\ UNCHANGED <<task, evSpawn>>

\* commit_event_with_log for the event at index e.pc, plus whatever else
\* the same database transaction does (`consume` notifications). The
\* INSERT only happens if running_on = us, otherwise we get
\* NotScheduledOnWorker.
\*
\* The running_on check uses the statement's snapshot, so an execution whose
\* worker row is being deleted concurrently can still insert the event after
\* the new owner has claimed the task. The model allows that stale insert
\* whenever the execution has lost the task, which is more often than in
\* reality. Whoever inserts second gets a primary key conflict, which
\* run_task treats as a lost race: reset for retry if we still own the task,
\* NotScheduledOnWorker otherwise.
\*
\* CommitLocksTask: current_task takes FOR SHARE on the task row. That
\* conflicts with the running_on = NULL update from deleting the worker row,
\* so the check sees the latest running_on and there is no stale insert.
CommitEvent(e, consume) ==
    LET take == IF consume > 0 THEN nq' = [nq EXCEPT ![e.t] = @ - consume]
                           ELSE UNCHANGED nq
    IN
    IF recorded[e.t] > e.pc
    THEN Finish(e) /\ ResetForRetry(e) /\ UNCHANGED <<nq, recorded>>
    ELSE IF Owns(e)
    THEN /\ take
         /\ RecordEvent(e.t)
         /\ Advance(e, [e EXCEPT !.pc = e.pc + 1, !.ph = "run"])
         /\ UNCHANGED <<task, evSpawn>>
    ELSE \/ Quit(e) /\ UNCHANGED <<nq, recorded>>
         \/ ~CommitLocksTask /\ take /\ RecordEvent(e.t) /\ Quit(e)

\* TaskState::exit -> commit_event_with_log, outside a database transaction.
ExecCommit(e) ==
    /\ e.ph = "commit"
    /\ CommitEvent(e, 0)
    /\ ExecUnchanged

\* wasi poll/block with a deadline beyond suspend_timeout:
\* TaskState::suspend -> suspend_task, which only applies if we still own
\* the task. If the task is already final its wasm is NULL and
\* check_wasm_while_active rejects the UPDATE; that error is not
\* recoverable, so the task fails.
ExecSleep(e) ==
    /\ e.ph = "sleep"
    /\ IF ~Owns(e)
       THEN Quit(e)
       ELSE IF task[e.t].state \in Final
       THEN Fail(e)
       ELSE /\ task' = [task EXCEPT ![e.t] = [state |-> "suspended",
                                              ro |-> Null, timer |-> TRUE]]
            /\ Finish(e)
            /\ UNCHANGED evSpawn
    /\ UNCHANGED <<nq, recorded>>
    /\ ExecUnchanged

\* The phase an execution waits in after an empty poll.
WaitPhase(e) == IF e.ph = "tpoll" THEN "twaiting" ELSE "waiting"
PollPhase(e) == IF e.ph = "twaiting" THEN "tpoll" ELSE "poll"

\* notification_blocking and notification_blocking_timeout: poll_notification
\* and the exit commit share one database transaction, so a notification is
\* consumed iff its event is recorded.
ExecPoll(e) ==
    /\ e.ph \in {"poll", "tpoll"}
    /\ IF nq[e.t] = 0
       THEN /\ Advance(e, [e EXCEPT !.ph = WaitPhase(e)])
            /\ UNCHANGED <<task, evSpawn, nq, recorded>>
       ELSE CommitEvent(e, 1)
    /\ ExecUnchanged

\* notification_blocking_timeout: the user's deadline has passed. Poll once
\* more; record the notification if there is one, and None otherwise.
ExecUserTimeout(e) ==
    /\ e.ph = "twaiting"
    /\ deadline[e.t]
    /\ CommitEvent(e, IF nq[e.t] > 0 THEN 1 ELSE 0)
    /\ ExecUnchanged

\* The wait hits suspend_timeout and calls try_suspend: lock the row, check
\* that the task is still active on this worker, suspend it, then poll once
\* more and roll back if a notification is there.
\*
\* A plain wait uses suspend_task_no_wakeup, which leaves wakeup_at alone.
\* A timed wait uses suspend_task with its recorded deadline, so the timer
\* revives it even if its notification never arrives. A timed wait only
\* gets here while its deadline is in the future: otherwise the user timeout
\* fires first (it is earlier in the biased select!).
ExecTimeout(e) ==
    /\ \/ e.ph = "waiting"
       \/ e.ph = "twaiting" /\ ~deadline[e.t]
    /\ IF ~Owns(e) \/ task[e.t].state # "active"
       THEN Quit(e)
       ELSE IF nq[e.t] > 0
       THEN Advance(e, [e EXCEPT !.ph = PollPhase(e)])
            /\ UNCHANGED <<task, evSpawn>>
       ELSE /\ task' = [task EXCEPT ![e.t].state = "suspended",
                                    ![e.t].ro = Null,
                                    ![e.t].timer = (e.ph = "twaiting" \/ @)]
            /\ Finish(e)
            /\ UNCHANGED evSpawn
    /\ UNCHANGED <<nq, recorded>>
    /\ ExecUnchanged

\* TaskStatus::ExitFailure -> mark_task_failed, which only applies if we
\* still own the task.
ExecFail(e) ==
    /\ e.ph = "error"
    /\ IF Owns(e)
       THEN task' = [task EXCEPT ![e.t].state = "failed", ![e.t].ro = Null]
       ELSE UNCHANGED task
    /\ Finish(e)
    /\ UNCHANGED <<nq, recorded, evSpawn>>
    /\ ExecUnchanged

\* A recoverable sqlx error (Io, PoolTimedOut, WorkerCrashed) anywhere in
\* the task: run_task calls reset_task_for_retry and returns.
ExecTransient(e) ==
    /\ transient < MaxTransient
    /\ transient' = transient + 1
    /\ Finish(e)
    /\ ResetForRetry(e)
    /\ UNCHANGED <<workerVars, evNotif, evWorker, evLagged, nq, sent,
                   recorded, deadline, late, reconnects>>

ExecStep(e) ==
    \/ ExecEnter(e)
    \/ ExecComplete(e)
    \/ ExecCommit(e)
    \/ ExecSleep(e)
    \/ ExecPoll(e)
    \/ ExecUserTimeout(e)
    \/ ExecTimeout(e)
    \/ ExecFail(e)

-----------------------------------------------------------------------------
(* The specification *)

Init ==
    /\ wstatus = [w \in Workers |-> "unborn"]
    /\ rows = {}
    /\ expired = {}
    /\ following = [w \in Workers |-> Null]
    /\ leader = [w \in Workers |-> Null]
    /\ blocked = [w \in Workers |-> FALSE]
    /\ evSpawn = [w \in Workers |-> FALSE]
    /\ evNotif = [w \in Workers |-> {}]
    /\ evWorker = [w \in Workers |-> {}]
    /\ evLagged = [w \in Workers |-> FALSE]
    /\ joinWake = [w \in Workers |-> FALSE]
    /\ task = [t \in Tasks |-> [state |-> "none", ro |-> Null, timer |-> FALSE]]
    /\ nq = [t \in Tasks |-> 0]
    /\ sent = [t \in Tasks |-> 0]
    /\ recorded = [t \in Tasks |-> 0]
    /\ deadline = [t \in Tasks |-> FALSE]
    /\ execs = {}
    /\ late = 0
    /\ transient = 0
    /\ reconnects = 0

WorkerNext(w) ==
    \/ Start(w)
    \/ Crash(w)
    \/ Expire(w)
    \/ LateHeartbeat(w)
    \/ HeartbeatRefresh(w)
    \/ HeartbeatDetect(w)
    \/ Validate(w)
    \/ LeaderWake(w)
    \/ Unwedge(w)
    \/ ProcessJoin(w)
    \/ ProcessSpawn(w)
    \/ ProcessLagged(w)
    \/ Reconnect(w)
    \/ \E t \in Tasks : ProcessNotif(w, t)
    \/ \E v \in Workers : ProcessWorker(w, v)

Next ==
    \/ \E w \in Workers : WorkerNext(w)
    \/ \E t \in Tasks : Launch(t) \/ SendNotify(t, NumSends) \/ Tick(t)
    \/ \E e \in execs : ExecStep(e) \/ ExecTransient(e)

\* Everything the system does on its own is weakly fair. Faults (Crash,
\* LateHeartbeat, Reconnect, ExecTransient) are not: they may happen, up to
\* their budgets, but never have to.
Fairness ==
    /\ \A w \in Workers :
          /\ WF_vars(Start(w))
          /\ WF_vars(Expire(w))
          /\ WF_vars(HeartbeatRefresh(w))
          /\ WF_vars(HeartbeatDetect(w))
          /\ WF_vars(Validate(w))
          /\ WF_vars(LeaderWake(w))
          /\ WF_vars(Unwedge(w))
          /\ WF_vars(ProcessJoin(w))
          /\ WF_vars(ProcessSpawn(w))
          /\ WF_vars(ProcessLagged(w))
          /\ \A t \in Tasks : WF_vars(ProcessNotif(w, t))
          /\ \A v \in Workers : WF_vars(ProcessWorker(w, v))
    /\ \A t \in Tasks :
          /\ WF_vars(Launch(t))
          \* Only the notifications for plain waits have to be sent.
          /\ WF_vars(SendNotify(t, NumWaits))
          /\ WF_vars(Tick(t))
    /\ \A w \in Workers, t \in Tasks :
          WF_vars(\E e \in execs : e.w = w /\ e.t = t /\ ExecStep(e))

Spec == Init /\ [][Next]_vars /\ Fairness

-----------------------------------------------------------------------------
(* Properties *)

\* Workflows never fail on their own, so a failed task is a runtime bug.
NoSpuriousFailure == \A t \in Tasks : task[t].state # "failed"

\* An execution is never ahead of the event log.
ExecsBehindLog == \A e \in execs : e.pc <= recorded[e.t]

\* Once a task is complete or failed it stays that way.
FinalIsFinal ==
    [][\A t \in Tasks : task[t].state \in Final => task'[t] = task[t]]_vars

\* The headline property: every task eventually completes.
AllTasksComplete == \A t \in Tasks : <>(task[t].state = "complete")

\* Weaker: every task eventually reaches some final state.
AllTasksFinish == \A t \in Tasks : <>(task[t].state \in Final)

\* The rows of dead workers are eventually reaped.
DeadRowsReaped ==
    \A w \in Workers : [](wstatus[w] = "dead" => <>(w \notin rows))

=============================================================================
