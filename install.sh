#!/usr/bin/env bash
set -e

REPO="IvanG11/bolt"
DEST="/usr/local/bin/bolt"
TMP="$(mktemp)"

echo "Installing bolt..."

curl -fsSL "https://github.com/$REPO/releases/latest/download/bolt" -o "$TMP"
install -m755 "$TMP" "$DEST"
rm -f "$TMP"

echo "bolt installed at $DEST"
