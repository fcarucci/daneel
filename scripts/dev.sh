#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0

set -euo pipefail

PORT="${1:-4127}"
ADDR="${2:-127.0.0.1}"

cleanup() {
  local exit_code=$?
  if [[ -n "${APP_PID:-}" ]]; then
    kill "${APP_PID}" 2>/dev/null || true
  fi
  if [[ -n "${CSS_PID:-}" ]]; then
    kill "${CSS_PID}" 2>/dev/null || true
  fi
  wait "${APP_PID:-}" 2>/dev/null || true
  wait "${CSS_PID:-}" 2>/dev/null || true
  exit "${exit_code}"
}

trap cleanup EXIT INT TERM

fuser -k "${PORT}/tcp" 2>/dev/null || true
npm run build:css

node scripts/watch-css.mjs &
CSS_PID=$!

dx serve --web --fullstack --addr "${ADDR}" --port "${PORT}" --open false &
APP_PID=$!

for _ in $(seq 1 120); do
  if curl -fsS "http://${ADDR}:${PORT}" >/dev/null 2>&1; then
    printf '[app] READY http://%s:%s\n' "${ADDR}" "${PORT}"
    wait "${APP_PID}"
    exit $?
  fi

  if ! kill -0 "${APP_PID}" 2>/dev/null; then
    printf '[app] ERROR dev server exited before becoming reachable\n' >&2
    wait "${APP_PID}" || true
    exit 1
  fi

  sleep 1
done

printf '[app] ERROR timed out waiting for http://%s:%s\n' "${ADDR}" "${PORT}" >&2
exit 1
