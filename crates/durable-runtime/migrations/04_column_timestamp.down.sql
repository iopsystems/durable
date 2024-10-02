-- Modify "event" table
ALTER TABLE "durable"."event" DROP COLUMN "created_at";
-- Modify "log" table
ALTER TABLE "durable"."log" DROP COLUMN "created_at";
