-- Modify "notify_notification" function
CREATE OR REPLACE FUNCTION "durable"."notify_notification" () RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
        PERFORM pg_notify(
            'durable:notification',
            jsonb_build_object(
                'task_id', NEW.task_id,
                'event', NEW.event
            )::text
        );

        -- Wake up the related task when a notification occurs.
        UPDATE durable.task
        SET state = 'ready',
            wakeup_at = NULL,
            running_on = (
                SELECT id
                 FROM durable.worker
                ORDER BY random()
                FOR SHARE SKIP LOCKED
            )
        WHERE id = NEW.task_id
          AND state = 'suspended';

        RETURN NULL;
    END;
$$;
