#!/bin/bash
set -euo pipefail

# Only run in remote (Claude Code on the web) environments.
if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

# ── Install cargo-nextest ────────────────────────────────────────────────
if ! command -v cargo-nextest &>/dev/null; then
  echo "Installing cargo-nextest..."
  curl -LsSf https://get.nexte.st/latest/linux -o /tmp/nextest.tar.gz
  tar xzf /tmp/nextest.tar.gz -C ~/.cargo/bin/
  rm /tmp/nextest.tar.gz
fi

# ── Install cargo-component (needed to build WASM test workflows) ────────
if ! command -v cargo-component &>/dev/null; then
  echo "Installing cargo-component..."
  cargo install cargo-component --locked
fi

# ── Ensure the wasm32-wasip1 target is available ────────────────────────
if ! rustup target list --installed | grep -q wasm32-wasip1; then
  echo "Adding wasm32-wasip1 target..."
  rustup target add wasm32-wasip1
fi

# ── Initialize the database schema ───────────────────────────────────────
# Starts PostgreSQL, configures trust auth, runs migrations, writes .env.
echo "Setting up the database..."
cd "$CLAUDE_PROJECT_DIR"
cargo xtask claude setup
