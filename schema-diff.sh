#!/usr/bin/bash

set -eux

rm -rf target/schema
rm -f target/migration.up.sql
rm -f target/migration.down.sql

mkdir -p target/schema

cp crates/durable-runtime/migrations/*.up.sql target/schema/
atlas migrate hash --dir file://target/schema

atlas schema diff                                               \
    --from file://target/schema                                 \
    --to   file://crates/durable-runtime/schema.sql             \
    --dev-url 'docker://postgres/15/test'                       \
    > target/migration.up.sql

atlas schema diff                                               \
    --to   file://target/schema                                 \
    --from file://crates/durable-runtime/schema.sql             \
    --dev-url 'docker://postgres/15/test'                       \
    > target/migration.down.sql
