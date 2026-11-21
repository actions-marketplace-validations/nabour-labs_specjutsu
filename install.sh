#!/bin/sh
# Install sj, the specjutsu CLI, from a GitHub release (macOS and Linux).
#
#   curl -fsSL https://raw.githubusercontent.com/nabour-labs/specjutsu/main/install.sh | sh
#
# Environment:
#   SJ_VERSION       release to install, e.g. 0.4.0 (default: the latest release)
#   SJ_INSTALL_DIR   where to put sj (default: $HOME/.local/bin)
#   SJ_DOWNLOAD_URL  folder holding the release assets (default: GitHub releases)
#
# The archive is checked against the release's SHA256SUMS before anything is installed.
# Windows: download the zip from the releases page, or run `cargo install specjutsu`.
set -eu

repo=https://github.com/nabour-labs/specjutsu

fail() {
  echo "install.sh: $*" >&2
  exit 1
}

os=$(uname -s)
arch=$(uname -m)
case "$os $arch" in
  "Darwin arm64" | "Darwin aarch64") target=aarch64-apple-darwin ;;
  "Darwin x86_64") target=x86_64-apple-darwin ;;
  "Linux x86_64" | "Linux amd64") target=x86_64-unknown-linux-musl ;;
  "Linux aarch64" | "Linux arm64") target=aarch64-unknown-linux-gnu ;;
  *) fail "no prebuilt sj for $os $arch; install from source with: cargo install specjutsu" ;;
esac

if [ -n "${SJ_DOWNLOAD_URL:-}" ]; then
  base=$SJ_DOWNLOAD_URL
elif [ -n "${SJ_VERSION:-}" ]; then
  base="$repo/releases/download/v${SJ_VERSION#v}"
else
  base="$repo/releases/latest/download"
fi
dir=${SJ_INSTALL_DIR:-$HOME/.local/bin}
asset="sj-$target.tar.gz"

command -v curl >/dev/null 2>&1 || fail "curl is required"
command -v tar >/dev/null 2>&1 || fail "tar is required"
if command -v sha256sum >/dev/null 2>&1; then
  sum() { sha256sum "$1" | cut -d' ' -f1; }
elif command -v shasum >/dev/null 2>&1; then
  sum() { shasum -a 256 "$1" | cut -d' ' -f1; }
else
  fail "sha256sum or shasum is required"
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT INT TERM

echo "downloading $base/$asset"
curl -fsSL "$base/$asset" -o "$tmp/$asset" || fail "cannot download $base/$asset"
curl -fsSL "$base/SHA256SUMS" -o "$tmp/SHA256SUMS" || fail "cannot download $base/SHA256SUMS"

want=$(awk -v a="$asset" '$2 == a || $2 == "*" a { print $1; exit }' "$tmp/SHA256SUMS")
[ -n "$want" ] || fail "no checksum for $asset in SHA256SUMS; not installing"
got=$(sum "$tmp/$asset")
[ "$want" = "$got" ] || fail "checksum mismatch for $asset (expected $want, got $got); not installing"

tar -xzf "$tmp/$asset" -C "$tmp"
mkdir -p "$dir"
cp "$tmp/sj-$target/sj" "$dir/sj.tmp"
chmod 755 "$dir/sj.tmp"
mv "$dir/sj.tmp" "$dir/sj"

echo "installed $dir/sj"
"$dir/sj" --version

case ":$PATH:" in
  *":$dir:"*) ;;
  *) echo "add $dir to your PATH, e.g. export PATH=\"$dir:\$PATH\"" ;;
esac
