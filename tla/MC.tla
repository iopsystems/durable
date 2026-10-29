---------------------------------- MODULE MC ----------------------------------
(* Definitions the .cfg files refer to. TLC config files cannot spell
   sequences portably, so they are defined here. *)
EXTENDS Durable

\* Wait for a notification, then sleep long enough to suspend.
WaitThenSleep == <<"wait", "sleep">>

\* notification_blocking_timeout: record the deadline, then wait for a
\* notification until it passes.
TimedWait == <<"txn", "twait">>
===============================================================================
