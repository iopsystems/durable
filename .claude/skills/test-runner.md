---
name: running-tests
description: Runs the project test suite using cargo-nextest, including unit and integration tests. Use when the user asks to run tests, verify changes, or validate before committing.
---

# Running Tests

## Prerequisites

If the environment is not set up, run `cargo xtask claude setup`.

## Full test suite

```sh
cargo nextest run --locked --all-targets --all-features --no-fail-fast
```

Nextest automatically builds WASM test binaries via `.config/nextest.toml` setup scripts before running tests.

## Specific test or crate

```sh
cargo nextest run --locked --all-features test_name
cargo nextest run --locked --all-features -p crate-name
```

## Key details

- Integration tests use `#[sqlx::test]` and require `DATABASE_URL` to be set (the session-start hook writes `.env`).
- The nextest setup script runs `cargo component build --profile wasm -p durable-test-workflows` to build WASM binaries loaded by tests at runtime.
- CI uses the `dev-ci` cargo profile. Add `--cargo-profile dev-ci` for the same behavior locally.
