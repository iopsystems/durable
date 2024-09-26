-- Modify "task_updated" trigger
CREATE OR REPLACE TRIGGER "task_updated"
    AFTER UPDATE OF "state" ON "durable"."task"
    FOR EACH ROW WHEN (
        (new.state = ANY (ARRAY['active'::durable.task_state, 'ready'::durable.task_state]))
        AND
        (NOT (old.state = ANY (ARRAY['active'::durable.task_state, 'ready'::durable.task_state])))
    )
    EXECUTE FUNCTION "durable"."notify_task"();
