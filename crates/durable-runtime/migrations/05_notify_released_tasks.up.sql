-- Notify workers when a task is released by the worker that was running it.
--
-- `fk_worker` is `ON DELETE SET NULL`, so deleting a worker row - which is
-- what reaping an expired worker does - hands its tasks back by clearing
-- `running_on` while leaving `state` alone. `task_updated` only fires on a
-- state transition into 'active'/'ready', so that release went unannounced and
-- no live worker learned there was a task to claim.
CREATE TRIGGER task_released
    AFTER UPDATE OF running_on ON durable.task
    FOR EACH ROW WHEN (
        NEW.running_on IS NULL
        AND
        NEW.state IN ('active', 'ready')
    )
    EXECUTE FUNCTION durable.notify_task();
