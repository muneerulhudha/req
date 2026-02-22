#!/usr/bin/env bash

set -euo pipefail

OWNER="${REQ_INSTALL_OWNER:-muneerulhudha}"
REPO="${REQ_INSTALL_REPO:-req}"
INSTALL_DIR="${REQ_INSTALL_DIR:-/usr/local/bin}"

ARCH="$(uname -m)"
case "$ARCH" in
  arm64)
    ASSET="req-aarch64-apple-darwin.tar.gz"
    ;;
  x86_64)
    ASSET="req-x86_64-apple-darwin.tar.gz"
    ;;
  *)
    echo "Unsupported architecture: $ARCH" >&2
    echo "This installer currently supports macOS arm64 and x86_64 only." >&2
    exit 1
    ;;
esac

URL="https://github.com/${OWNER}/${REPO}/releases/latest/download/${ASSET}"
TMP_DIR="$(mktemp -d)"
ARCHIVE_PATH="${TMP_DIR}/req.tar.gz"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

echo "Downloading ${URL}"
curl -fL "$URL" -o "$ARCHIVE_PATH"

mkdir -p "$INSTALL_DIR"
if [ -w "$INSTALL_DIR" ]; then
  tar -xzf "$ARCHIVE_PATH" -C "$INSTALL_DIR" req
else
  sudo tar -xzf "$ARCHIVE_PATH" -C "$INSTALL_DIR" req
fi

if [ -w "${INSTALL_DIR}/req" ]; then
  chmod +x "${INSTALL_DIR}/req"
else
  sudo chmod +x "${INSTALL_DIR}/req"
fi

echo "Installed req to ${INSTALL_DIR}/req"
echo "Run 'req --help' to verify."
