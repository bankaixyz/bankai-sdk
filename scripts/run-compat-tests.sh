#!/usr/bin/env bash
set -euo pipefail

export COMPAT_API_BASE_URL="${COMPAT_API_BASE_URL:-http://127.0.0.1:8081}"
export COMPAT_VERBOSE="${COMPAT_VERBOSE:-0}"
export COMPAT_COLOR="${COMPAT_COLOR:-1}"

echo "Running SDK compatibility tests against ${COMPAT_API_BASE_URL}"

cargo test -p bankai-sdk --test compat_live -- --ignored --nocapture
