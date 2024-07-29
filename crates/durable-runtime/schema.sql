
CREATE TABLE worker(
    id              bigserial   NOT NULL PRIMARY KEY,
    last_heartbeat  timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX worker_timestamp ON worker(last_heartbeat ASC);

CREATE TYPE task_state AS ENUM(
    'active',
    'complete',
    'failed'
);

CREATE TABLE task(
    id              bigserial   NOT NULL PRIMARY KEY,
    name            text        NOT NULL,
    state           task_state  NOT NULL DEFAULT 'active',
    running_on      bigint,

    created_at      timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at    timestamptz,

    -- The compiled WASM bytecode.
    --
    -- This gets set to NULL once the task has executed to completion.
    wasm            bytea,
    data            jsonb       NOT NULL,

    CONSTRAINT fk_worker FOREIGN KEY(running_on) REFERENCES worker(id)
        ON DELETE SET NULL
);

CREATE INDEX task_queue ON task(running_on ASC)
    WHERE state = 'active';

CREATE TABLE event(
    task_id         bigint      NOT NULL,
    index           int         NOT NULL,

    -- A user-visible text label. Used for debugging and for ensuring that
    -- the events requested are consistent.
    label           text        NOT NULL,

    -- The actual value that was returned by the transaction.
    value           jsonb       NOT NULL,

    PRIMARY KEY(task_id, index),

    CONSTRAINT fk_task FOREIGN KEY(task_id) REFERENCES task(id)
        ON DELETE CASCADE
);

CREATE TABLE notification(
    task_id         bigint      NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,

    event           text        NOT NULL,
    data            jsonb       NOT NULL,

    CONSTRAINT fk_task FOREIGN KEY(task_id) REFERENCES task(id)
);

CREATE TABLE logs(
    task_id         bigint      NOT NULL,
    index           int         NOT NULL,
    message         text        NOT NULL,

    PRIMARY KEY(task_id, index),

    CONSTRAINT fk_task  FOREIGN KEY(task_id) REFERENCES task(id)
        ON DELETE CASCADE,
    CONSTRAINT fk_event FOREIGN KEY(task_id, index) REFERENCES event(task_id, index)
        ON DELETE CASCADE
);

CREATE FUNCTION notify_task_inserted() RETURNS trigger as $$
    BEGIN
        PERFORM pg_notify(
            'durable:task-inserted',
            jsonb_build_object(
                'id', NEW.id,
                'running_on', NEW.running_on
            )::text
        );
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION notify_notification_inserted() RETURNS trigger as $$
    BEGIN
        PERFORM pg_notify(
            'durable:notification-inserted',
            jsonb_build_object(
                'task_id', NEW.task_id,
                'event', NEW.event
            )::text
        );
        RETURN NEW;
    END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER task_inserted
    AFTER INSERT ON task
    FOR EACH ROW EXECUTE PROCEDURE notify_task_inserted();

CREATE TRIGGER task_updated
    AFTER UPDATE OF running_on ON task
    FOR EACH ROW WHEN (NEW.running_on IS NULL AND NEW.state = 'active')
    EXECUTE PROCEDURE notify_task_inserted();

CREATE TRIGGER notification_inserted
    AFTER INSERT ON notification
    FOR EACH ROW EXECUTE PROCEDURE notify_notification_inserted();
