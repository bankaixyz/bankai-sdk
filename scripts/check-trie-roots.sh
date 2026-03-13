#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 3 || $# -gt 4 ]]; then
  echo "usage: $0 <execution|op-stack> <rpc-url> <start-block> [count]" >&2
  exit 1
fi

cargo run -p bankai-core --bin check-trie-roots -- "$@"
