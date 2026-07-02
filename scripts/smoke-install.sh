#!/usr/bin/env sh
set -eu

REPO_ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
VERSION="${SLOP_LINT_VERSION:-0.1.0}"
TMPDIR="${TMPDIR:-/tmp}"

make_tmpdir() {
  mktemp -d "$TMPDIR/slop-lint-smoke.XXXXXX"
}

assert_contains() {
  haystack="$1"
  needle="$2"
  label="$3"
  if ! printf '%s' "$haystack" | grep -F "$needle" >/dev/null; then
    echo "smoke-install: expected $label to contain: $needle" >&2
    exit 1
  fi
}

run_binary_smoke() {
  bin="$1"
  workspace="$2"

  version_output="$("$bin" --version)"
  assert_contains "$version_output" "slop-lint $VERSION" "version output"

  printf 'It is important to note that this is robust and seamless.\n' > "$workspace/english.md"
  english_output="$("$bin" check "$workspace/english.md" --format json)"
  assert_contains "$english_output" '"rule_id": "slop.overconfident-transition"' "English diagnostic output"

  printf '值得注意的是，这是一套全面、强大、高效、创新、无缝的方案。\n' > "$workspace/chinese.md"
  chinese_output="$("$bin" check "$workspace/chinese.md" --format json)"
  assert_contains "$chinese_output" '"rule_id": "slop.zh-formulaic-transition"' "Chinese diagnostic output"
}

script_tmpdir="$(make_tmpdir)"
SLOP_LINT_VERSION="v$VERSION" SLOP_LINT_INSTALL_DIR="$script_tmpdir/bin" sh "$REPO_ROOT/install.sh"
run_binary_smoke "$script_tmpdir/bin/slop-lint" "$script_tmpdir"
echo "smoke-install: install.sh path passed"

if ! command -v cargo >/dev/null 2>&1; then
  echo "smoke-install: cargo is required for the crates.io install smoke" >&2
  exit 1
fi

cargo_tmpdir="$(make_tmpdir)"
cargo install slop-lint --version "$VERSION" --root "$cargo_tmpdir" --locked
run_binary_smoke "$cargo_tmpdir/bin/slop-lint" "$cargo_tmpdir"
echo "smoke-install: cargo install path passed"

echo "smoke-install: all checks passed"
