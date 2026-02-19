---
name: running-tests
description: Runs the project test suite using cargo-nextest, including unit tests, integration tests, doc tests, formatting checks, and clippy. Use when the user asks to run tests, check code quality, verify changes, or validate before committing.
---

# Running Tests

## Prerequisites

If the environment is not set up, run `cargo xtask claude setup`.

## Full test suite

```sh
cargo nextest run --locked --all-targets --all-features --no-fail-fast
```

Nextest automatically builds WASM test binaries via `.config/nextest.toml` setup scripts before running tests.

## Doc tests

Nextest does not support doc tests; run them separately:

```sh
cargo test --doc --all-features --locked
```

## Specific test or crate

```sh
cargo nextest run --locked --all-features test_name
cargo nextest run --locked --all-features -p crate-name
```

## Formatting and clippy

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features
```

## Key details

- Integration tests use `#[sqlx::test]` and require `DATABASE_URL` to be set (the session-start hook writes `.env`).
- The nextest setup script runs `cargo component build --profile wasm -p durable-test-workflows` to build WASM binaries loaded by tests at runtime.
- CI uses the `dev-ci` cargo profile. Add `--cargo-profile dev-ci` for the same behavior locally.
