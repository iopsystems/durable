#!/bin/bash
set -euo pipefail

# Only run in remote (Claude Code on the web) environments.
if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

# Install tools (cargo-nextest, cargo-component, wasm32-wasip1 target),
# start PostgreSQL, run migrations, and write the .env file.
cd "$CLAUDE_PROJECT_DIR"
cargo xtask claude setup
