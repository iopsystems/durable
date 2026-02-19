# Test Runner

Use this skill to run the project's test suite. This is a Rust workspace that uses
`cargo-nextest` as the test runner, with tests that require a PostgreSQL database
and pre-built WebAssembly binaries.

## Prerequisites

The SessionStart hook (`.claude/hooks/session-start.sh`) installs required tools and
starts PostgreSQL. If the environment is not set up, run:

```sh
cargo xtask claude setup
```

This ensures `cargo-nextest`, `cargo-component`, the `wasm32-wasip1` target, and a
PostgreSQL database are all available.

## Running the full test suite

Run all tests (unit, integration, and deterministic simulation tests):

```sh
cargo nextest run --locked --all-targets --all-features --no-fail-fast
```

The nextest configuration in `.config/nextest.toml` automatically triggers a setup
script that builds the required WASM test binaries before running tests that depend
on them.

## Running doc tests

Doc tests are not supported by nextest and must be run separately:

```sh
cargo test --doc --all-features --locked
```

## Running a specific test

Pass a filter expression or test name to nextest:

```sh
cargo nextest run --locked --all-features -E 'test(test_name)'
```

Or use a simple string filter:

```sh
cargo nextest run --locked --all-features test_name
```

## Running tests for a specific crate

```sh
cargo nextest run --locked --all-features -p crate-name
```

## Checking formatting

```sh
cargo fmt --all -- --check
```

## Running clippy

```sh
cargo clippy --all-targets --all-features
```

## Key details

- Tests in `crates/durable-test/` use `#[sqlx::test]` and require a running
  PostgreSQL instance. The `DATABASE_URL` environment variable must be set (the
  session-start hook writes this to `.env`).
- The nextest setup script (`crates/durable-test/setup.sh`) builds WASM binaries
  from `crates/durable-test-workflows/` with `cargo component build --profile wasm`.
  These binaries are loaded by integration tests at runtime.
- CI uses the `dev-ci` cargo profile for faster builds. You can add
  `--cargo-profile dev-ci` to nextest commands locally for the same behavior.
