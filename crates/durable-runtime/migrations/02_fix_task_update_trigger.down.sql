-- Modify "task_updated" trigger
CREATE OR REPLACE TRIGGER "task_updated"
    AFTER UPDATE OF "running_on" ON "durable"."task"
    FOR EACH ROW WHEN ((new.running_on IS NULL) AND (new.state = ANY (ARRAY['active'::durable.task_state, 'ready'::durable.task_state])))
    EXECUTE FUNCTION "durable"."notify_task"();
