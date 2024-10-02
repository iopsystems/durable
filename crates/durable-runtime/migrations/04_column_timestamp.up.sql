-- Modify "event" table
ALTER TABLE "durable"."event" ADD COLUMN "created_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP;
-- Modify "log" table
ALTER TABLE "durable"."log" ADD COLUMN "created_at" timestamptz NOT NULL DEFAULT CURRENT_TIMESTAMP;
