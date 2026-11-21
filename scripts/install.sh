#!/bin/sh
# Install sj, the specjutsu CLI, on macOS or Linux (DIST-001..005).
#
#   curl -fsSL https://nabour-labs.github.io/specjutsu/install.sh | sh
#
# Environment:
#   SJ_VERSION        release to install, e.g. 0.4.0 (default: the latest release)
#   SJ_INSTALL_DIR    folder for the sj binary (default: ~/.local/bin)
#   SJ_DOWNLOAD_BASE  where releases live (default: https://github.com/nabour-labs/specjutsu/releases)
#
# The archive is checked against the release's SHA256SUMS before anything is installed.
# Shell profiles are never edited: if the folder isn't on PATH, the line to add is printed.
set -eu

base="${SJ_DOWNLOAD_BASE:-https://github.com/nabour-labs/specjutsu/releases}"
dest="${SJ_INSTALL_DIR:-$HOME/.local/bin}"

err() { echo "sj install: $*" >&2; }

os=$(uname -s)
arch=$(uname -m)
case "$os/$arch" in
  Darwin/arm64 | Darwin/aarch64) target=aarch64-apple-darwin ;;
  Darwin/x86_64) target=x86_64-apple-darwin ;;
  Linux/x86_64 | Linux/amd64) target=x86_64-unknown-linux-musl ;;
  Linux/aarch64 | Linux/arm64) target=aarch64-unknown-linux-gnu ;;
  *)
    err "no sj release for $os/$arch"
    err "build it from source instead: cargo install specjutsu"
    exit 2
    ;;
esac

if [ -n "${SJ_VERSION:-}" ]; then
  url="$base/download/v${SJ_VERSION#v}"
else
  url="$base/latest/download"
fi
asset="sj-$target.tar.gz"

fetch() {
  if command -v curl > /dev/null 2>&1; then
    curl --proto '=https,file' -fsSL "$1" -o "$2"
  elif command -v wget > /dev/null 2>&1; then
    wget -q "$1" -O "$2"
  else
    err "needs curl or wget"
    exit 1
  fi
}

sha256() {
  if command -v sha256sum > /dev/null 2>&1; then
    sha256sum "$1" | cut -d' ' -f1
  else
    shasum -a 256 "$1" | cut -d' ' -f1
  fi
}

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT INT TERM

echo "downloading $asset from $url"
fetch "$url/$asset" "$tmp/$asset" || { err "could not download $url/$asset"; exit 1; }
fetch "$url/SHA256SUMS" "$tmp/SHA256SUMS" || { err "could not download $url/SHA256SUMS"; exit 1; }

want=$(awk -v a="$asset" '$2 == a || $2 == "*" a { print $1 }' "$tmp/SHA256SUMS")
have=$(sha256 "$tmp/$asset")
if [ -z "$want" ] || [ "$want" != "$have" ]; then
  err "checksum mismatch for $asset"
  exit 1
fi

tar -xzf "$tmp/$asset" -C "$tmp"
mkdir -p "$dest"
# Copy then rename, so a running sj is never left half-written.
cp "$tmp/sj-$target/sj" "$dest/.sj.new"
chmod 755 "$dest/.sj.new"
mv -f "$dest/.sj.new" "$dest/sj"
echo "installed sj to $dest/sj"

case ":$PATH:" in
  *":$dest:"*) ;;
  *)
    echo
    echo "$dest is not on your PATH. Add it to your shell profile:"
    echo "  export PATH=\"$dest:\$PATH\""
    ;;
esac
