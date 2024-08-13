
DROP FUNCTION durable.notify_worker;
DROP FUNCTION durable.notify_log;
DROP FUNCTION durable.notify_notification;
DROP FUNCTION durable.notify_task_completed;
DROP FUNCTION durable.notify_task_suspended;
DROP FUNCTION durable.notify_task;

DROP TABLE durable.log;
DROP TABLE durable.notification;
DROP TABLE durable.event;
DROP TABLE durable.task;
DROP TABLE durable.wasm;
DROP TABLE durable.worker;

DROP TYPE durable.task_state;

-- We don't want to drop the schema here since we store the migrations table within.
-- DROP SCHEMA durable;
