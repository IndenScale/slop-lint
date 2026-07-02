#!/usr/bin/env sh
set -eu

REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"

(
  cd "$REPO_ROOT/docs"
  bun run build
)

(
  cd "$REPO_ROOT"
  cargo fmt --check
  cargo clippy --all-targets -- -D warnings
  cargo test
  cargo run -- check README.md README_ZH.md CHANGELOG.md RELEASE.md docs --format text
)
