#!/usr/bin/env bash
# Install a released sj for this runner, verifying it against the release's SHA256SUMS (CI-007).
# Usage: install.sh <version>
# Env: GH_TOKEN (for gh), SJ_INSTALL_DIR (default $RUNNER_TEMP/sj-bin or ~/.local/bin),
#      SJ_TARGET (override the detected release target).
set -euo pipefail
version="${1:?usage: install.sh <version>}"

if [ -z "${SJ_TARGET:-}" ]; then
  os=$(uname -s)
  arch=$(uname -m)
  case "$os/$arch" in
    Linux/x86_64) SJ_TARGET=x86_64-unknown-linux-gnu ;;
    Linux/aarch64 | Linux/arm64) SJ_TARGET=aarch64-unknown-linux-gnu ;;
    Darwin/arm64) SJ_TARGET=aarch64-apple-darwin ;;
    Darwin/x86_64) SJ_TARGET=x86_64-apple-darwin ;;
    MINGW* | MSYS* | CYGWIN*) SJ_TARGET=x86_64-pc-windows-msvc ;;
    *) echo "sj: no release for $os/$arch" >&2; exit 2 ;;
  esac
fi
case "$SJ_TARGET" in
  *windows*) asset="sj-$SJ_TARGET.zip"; exe=sj.exe ;;
  *) asset="sj-$SJ_TARGET.tar.gz"; exe=sj ;;
esac

dir=$(mktemp -d)
gh release download "v$version" --repo nabour-labs/specjutsu --dir "$dir" \
  --pattern "$asset" --pattern SHA256SUMS

want=$(awk -v a="$asset" '$2 == a || $2 == "*" a { print $1 }' "$dir/SHA256SUMS")
if command -v sha256sum > /dev/null; then
  have=$(sha256sum "$dir/$asset" | cut -d' ' -f1)
else
  have=$(shasum -a 256 "$dir/$asset" | cut -d' ' -f1)
fi
if [ -z "$want" ] || [ "$want" != "$have" ]; then
  echo "checksum mismatch for $asset" >&2
  exit 1
fi

case "$asset" in
  *.zip) unzip -q "$dir/$asset" -d "$dir" ;;
  *) tar -xzf "$dir/$asset" -C "$dir" ;;
esac
dest="${SJ_INSTALL_DIR:-${RUNNER_TEMP:-$HOME/.local}/sj-bin}"
mkdir -p "$dest"
cp "$dir/sj-$SJ_TARGET/$exe" "$dest/$exe"
chmod +x "$dest/$exe"
echo "$dest" >> "${GITHUB_PATH:-/dev/null}"
"$dest/$exe" --version
