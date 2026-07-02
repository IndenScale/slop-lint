#!/usr/bin/env sh
set -eu

REPO="${SLOP_LINT_REPO:-IndenScale/slop-lint}"
VERSION="${SLOP_LINT_VERSION:-latest}"
INSTALL_DIR="${SLOP_LINT_INSTALL_DIR:-$HOME/.local/bin}"

os="$(uname -s | tr '[:upper:]' '[:lower:]')"
arch="$(uname -m)"

case "$os" in
  darwin) os="apple-darwin" ;;
  linux) os="unknown-linux-gnu" ;;
  *) echo "slop-lint: unsupported OS: $os" >&2; exit 1 ;;
esac

case "$arch" in
  arm64|aarch64) arch="aarch64" ;;
  x86_64|amd64) arch="x86_64" ;;
  *) echo "slop-lint: unsupported architecture: $arch" >&2; exit 1 ;;
esac

target="$arch-$os"
asset="slop-lint-$target.tar.gz"

if [ "$VERSION" = "latest" ]; then
  url="https://github.com/$REPO/releases/latest/download/$asset"
  sums_url="https://github.com/$REPO/releases/latest/download/SHA256SUMS"
else
  url="https://github.com/$REPO/releases/download/$VERSION/$asset"
  sums_url="https://github.com/$REPO/releases/download/$VERSION/SHA256SUMS"
fi

tmpdir="$(mktemp -d)"
cleanup() {
  rm -rf "$tmpdir"
}
trap cleanup EXIT

mkdir -p "$INSTALL_DIR"

echo "slop-lint: downloading $url"
curl -fsSL "$url" -o "$tmpdir/$asset"
curl -fsSL "$sums_url" -o "$tmpdir/SHA256SUMS"
grep "  $asset\$" "$tmpdir/SHA256SUMS" > "$tmpdir/SHA256SUMS.asset"
(
  cd "$tmpdir"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum -c SHA256SUMS.asset
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 -c SHA256SUMS.asset
  else
    echo "slop-lint: sha256sum or shasum is required to verify $asset" >&2
    exit 1
  fi
)
tar -xzf "$tmpdir/$asset" -C "$tmpdir"
install "$tmpdir/slop-lint" "$INSTALL_DIR/slop-lint"

echo "slop-lint: installed to $INSTALL_DIR/slop-lint"
if ! command -v slop-lint >/dev/null 2>&1; then
  echo "slop-lint: add $INSTALL_DIR to PATH if slop-lint is not found"
fi
