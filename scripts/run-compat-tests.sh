#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: ./scripts/run-compat-tests.sh [--profile coverage|full] [--target sepolia|localhost] [--api-base-url URL]

Examples:
  ./scripts/run-compat-tests.sh --profile coverage --target sepolia
  ./scripts/run-compat-tests.sh --profile full --target localhost
  ./scripts/run-compat-tests.sh --profile coverage --api-base-url http://127.0.0.1:8081
EOF
}

profile="${COMPAT_PROFILE:-coverage}"
target="sepolia"
api_base_url="${COMPAT_API_BASE_URL:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      profile="${2:-}"
      shift 2
      ;;
    --target)
      target="${2:-}"
      shift 2
      ;;
    --api-base-url)
      api_base_url="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

case "$profile" in
  coverage|full)
    ;;
  *)
    echo "Invalid profile: $profile" >&2
    usage >&2
    exit 1
    ;;
esac

if [[ -z "$api_base_url" ]]; then
  case "$target" in
    sepolia)
      api_base_url="https://sepolia.api.bankai.xyz"
      ;;
    localhost)
      api_base_url="http://127.0.0.1:8081"
      ;;
    *)
      echo "Invalid target: $target" >&2
      usage >&2
      exit 1
      ;;
  esac
fi

export COMPAT_PROFILE="$profile"
export COMPAT_API_BASE_URL="${api_base_url%/}"
export COMPAT_VERBOSE="${COMPAT_VERBOSE:-0}"
export COMPAT_COLOR="${COMPAT_COLOR:-1}"

if [[ "${COMPAT_API_BASE_URL}" == http://127.0.0.1* ]] || [[ "${COMPAT_API_BASE_URL}" == http://localhost* ]]; then
  export BANKAI_SDK_NO_PROXY=1
  export ALL_PROXY=""
  export HTTP_PROXY=""
  export HTTPS_PROXY=""
  export NO_PROXY="${NO_PROXY:-127.0.0.1,localhost}"
fi

echo "Running SDK compatibility tests"
echo "  profile: ${COMPAT_PROFILE}"
echo "  api: ${COMPAT_API_BASE_URL}"

run_suite() {
  local suite="$1"
  echo
  echo "==> ${suite}"
  cargo test -p bankai-sdk --test compat_live "$suite" -- --ignored --nocapture
}

run_suite compat_live_decode_suite
run_suite compat_live_verify_suite
run_suite compat_live_openapi_coverage
