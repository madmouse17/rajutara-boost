#!/bin/sh
# Handshake + tools/list + one tools/call against the release binary. Usage: ./check.sh [file-to-smoke]
BIN="$(dirname "$0")/target/release/optirust-mcp"
FILE="${1:-}"
{
  printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"check","version":"0"}}}'
  printf '%s\n' '{"jsonrpc":"2.0","method":"notifications/initialized"}'
  printf '%s\n' '{"jsonrpc":"2.0","id":2,"method":"tools/list"}'
  [ -n "$FILE" ] && printf '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"optirust_smoke","arguments":{"path":"%s"}}}\n' "$FILE"
  sleep 2
} | "$BIN"
