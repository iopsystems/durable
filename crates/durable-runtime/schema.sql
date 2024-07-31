
CREATE SCHEMA durable;

-- Active workers in this durable cluster.
--
-- This table is used for a few different purposes:
-- * By clients, to pick a random host on which to schedule new tasks.
-- * By workers, to remove hosts which are no longer active.
-- * By workers, to determine which worker is the current leader.
--
-- Note that regular cluster operation is not blocked on the leader
-- acknowledging anything. The leader role is only responsible for performing
-- periodic cluster-level cleanup of old unused data.
CREATE TABLE durable.worker(
    id              bigserial   NOT NULL PRIMARY KEY,
    -- The timestamp at which this worker was created.
    --
    -- This should _never_ be set externally, as that would cause multiple
    -- workers to think that they are the leader.
    started_at      timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    heartbeat_at    timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX worker_started   ON durable.worker(started_at ASC);
CREATE INDEX worker_heartbeat ON durable.worker(heartbeat_at ASC);

-- Wasm binaries for use by various tasks.
--
-- This allows the binary itself to be shared by multiple jobs, all of which
-- may be using it. Since WASM binaries tend to be rather large, (100s of KBs
-- or larger,) this can save a lot of space in the database when there are
-- lots of tasks flying around. 
CREATE TABLE durable.wasm(
    id          bigserial   NOT NULL PRIMARY KEY,

    -- A SHA256 hash of the the wasm program here.
    --
    -- This is to avoid storing duplicate wasm programs in the database.
    hash        bytea       NOT NULL CONSTRAINT hash_unique UNIQUE,
    wasm        bytea       NOT NULL,

    -- An optional name for this program.
    --
    -- This has no semantic meaning within the runtime. It is, however,
    -- available for monitoring and debugging purposes.
    name        text,

    -- The last time that this program was created by a client.
    --
    -- Clients will update this peridically on use. A row will only be
    -- automatically deleted from this table if there are no workflows that use
    -- it and its last_used timestamp is more than a day in the past.
    last_used   timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TYPE durable.task_state AS ENUM(
    'active',
    'suspended',
    'complete',
    'failed'
);

CREATE TABLE durable.task(
    id              bigserial   NOT NULL PRIMARY KEY,
    name            text        NOT NULL,
    state   durable.task_state  NOT NULL DEFAULT 'active',
    running_on      bigint,

    created_at      timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at    timestamptz,
    wakeup_at       timestamptz,

    -- The compiled WASM bytecode.
    --
    -- This gets set to NULL once the task has executed to completion.
    wasm            bigint,
    data            jsonb       NOT NULL,

    CONSTRAINT fk_worker FOREIGN KEY(running_on) REFERENCES durable.worker(id)
        ON DELETE SET NULL,
    CONSTRAINT fk_wasm   FOREIGN KEY(wasm)       REFERENCES durable.wasm(id),

    CONSTRAINT check_wasm_while_active CHECK (
        wasm IS NOT NULL OR (state IN ('complete', 'failed'))
    )
);

CREATE INDEX task_queue ON durable.task(running_on ASC)
    WHERE state = 'active';
CREATE INDEX task_wasm ON durable.task(wasm)
    WHERE wasm IS NOT NULL;
CREATE INDEX task_suspended ON durable.task(wakeup_at ASC NULLS FIRST)
    WHERE state = 'suspended';

CREATE TABLE durable.event(
    task_id         bigint      NOT NULL,
    index           int         NOT NULL,

    -- A user-visible text label. Used for debugging and for ensuring that
    -- the events requested are consistent.
    label           text        NOT NULL,

    -- The actual value that was returned by the transaction.
    value           jsonb       NOT NULL,

    PRIMARY KEY(task_id, index),

    CONSTRAINT fk_task FOREIGN KEY(task_id) REFERENCES durable.task(id)
        ON DELETE CASCADE
);

CREATE TABLE durable.notification(
    task_id         bigint      NOT NULL,
    created_at      timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP,

    event           text        NOT NULL,
    data            jsonb       NOT NULL,

    CONSTRAINT fk_task FOREIGN KEY(task_id) REFERENCES durable.task(id)
);

CREATE TABLE durable.log(
    task_id         bigint      NOT NULL,
    index           int         NOT NULL,
    message         text        NOT NULL,

    PRIMARY KEY(task_id, index),

    CONSTRAINT fk_task  FOREIGN KEY(task_id) REFERENCES durable.task(id)
        ON DELETE CASCADE
);

CREATE FUNCTION durable.notify_task() RETURNS trigger as $$
    BEGIN
        PERFORM pg_notify(
            'durable:task',
            jsonb_build_object(
                'id', NEW.id,
                'running_on', NEW.running_on
            )::text
        );
        RETURN NULL;
    END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION durable.notify_task_suspended() RETURNS trigger as $$
    BEGIN
        PERFORM pg_notify(
            'durable:task-suspend',
            jsonb_build_object('id', NEW.id)::text
        );
        RETURN NULL;
    END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION durable.notify_task_completed() RETURNS trigger as $$
    BEGIN
        PERFORM pg_notify(
            'durable:task-complete',
            jsonb_build_object(
                'id', NEW.id,
                'state', NEW.state
            )::text
        );
        RETURN NULL;
    END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION durable.notify_notification() RETURNS trigger as $$
    BEGIN
        PERFORM pg_notify(
            'durable:notification',
            jsonb_build_object(
                'task_id', NEW.task_id,
                'event', NEW.event
            )::text
        );
        RETURN NULL;
    END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION durable.notify_log() RETURNS trigger AS $$
    BEGIN
        PERFORM pg_notify(
            'durable:log',
            jsonb_build_object(
                'task_id', NEW.task_id,
                'index', NEW.index
            )::text
        );
        RETURN NULL;
    END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION durable.notify_worker() RETURNS trigger as $$
    DECLARE
        worker_id   bigint;
    BEGIN
        IF (TG_OP = 'DELETE') THEN
            worker_id = OLD.id;
        ELSE
            worker_id = NEW.id;
        END IF;

        PERFORM pg_notify(
            'durable:worker',
            jsonb_build_object('worker_id', worker_id)::text
        );
        return NULL;
    END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER task_inserted
    AFTER INSERT ON durable.task
    FOR EACH ROW EXECUTE FUNCTION durable.notify_task();

CREATE TRIGGER task_updated
    AFTER UPDATE OF running_on ON durable.task
    FOR EACH ROW WHEN (NEW.running_on IS NULL AND NEW.state = 'active')
    EXECUTE FUNCTION durable.notify_task();

CREATE TRIGGER task_suspended
    AFTER INSERT OR UPDATE OF state ON durable.task
    FOR EACH ROW WHEN (NEW.state = 'suspended')
    EXECUTE FUNCTION durable.notify_task_suspended();

CREATE TRIGGER task_completed
    AFTER INSERT OR UPDATE OF state ON durable.task
    FOR EACH ROW WHEN (NEW.state = 'complete' OR NEW.state = 'failed')
    EXECUTE FUNCTION durable.notify_task_completed();

CREATE TRIGGER notification_inserted
    AFTER INSERT ON durable.notification
    FOR EACH ROW EXECUTE FUNCTION durable.notify_notification();

CREATE TRIGGER worker_inserted
    AFTER INSERT ON durable.worker
    FOR EACH ROW EXECUTE FUNCTION durable.notify_worker();

CREATE TRIGGER worker_deleted
    AFTER DELETE ON durable.worker
    FOR EACH ROW EXECUTE FUNCTION durable.notify_worker();

CREATE TRIGGER logs_inserted
    AFTER INSERT ON durable.log
    FOR EACH ROW EXECUTE FUNCTION durable.notify_log();
