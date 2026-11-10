#!/usr/bin/env bash
# Runs the whole app for development: backend, then frontend. Ctrl+C stops both.
set -euo pipefail
cd "$(dirname "$0")"

readonly ROOT=$PWD
readonly BACKEND_BIN=$ROOT/backend/target/release/ieftin-backend
readonly VITE_BIN=$ROOT/frontend/node_modules/vite/bin/vite.js
readonly PID_FILE=$ROOT/.start.pid

[[ $# -eq 0 ]] || { echo "Usage: $0" >&2; exit 2; }

step() { printf '\n\033[1m▸ %s\033[0m\n' "$1"; }
fail() { echo "$1" >&2; exit 1; }

# --- Prerequisites ----------------------------------------------------------
# rustup installs cargo here; not every shell profile puts it on PATH.
[[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"
for tool in cargo npm curl; do
  command -v "$tool" >/dev/null || fail "$tool is required."
done

# Each side has its own settings file, created from its example on first run.
for side in backend frontend; do
  [[ -f $side/.env ]] || { cp "$side/.env.example" "$side/.env"; echo "Created $side/.env from $side/.env.example"; }
done
set -a; source backend/.env; source frontend/.env; set +a

# The dev server forwards /api to wherever the backend listens, so the two
# cannot drift apart.
: "${BIND_ADDR:=127.0.0.1:${PORT:-8090}}" "${FRONTEND_PORT:=5180}"
export BIND_ADDR FRONTEND_PORT BACKEND_URL=http://$BIND_ADDR

# --- Process management -----------------------------------------------------
pids=()

# Stops this project's backend and frontend, wherever they were started from.
stop_services() {
  [[ ${#pids[@]} -eq 0 ]] || kill "${pids[@]}" 2>/dev/null || true
  pkill -x "$(basename "$BACKEND_BIN")" 2>/dev/null || true
  pkill -f "$VITE_BIN" 2>/dev/null || true
  wait 2>/dev/null || true
}

cleanup() {
  trap - EXIT INT TERM HUP
  step 'Stopping'
  stop_services
  rm -f "$PID_FILE"
  echo 'Everything stopped.'
}

port_is_free() { ! (exec 3<>"/dev/tcp/127.0.0.1/$1") 2>/dev/null; }

# --- Start ------------------------------------------------------------------
step 'Cleaning up previous run'
# Ask a start.sh that is still running to shut down, and wait until it has, so its
# cleanup cannot take down what this run is about to start.
if [[ -f $PID_FILE ]] && previous=$(<"$PID_FILE") && kill -0 "$previous" 2>/dev/null; then
  kill "$previous"
  while kill -0 "$previous" 2>/dev/null; do sleep 0.2; done
fi
echo $$ >"$PID_FILE"
stop_services
for port in "${BIND_ADDR##*:}" "$FRONTEND_PORT"; do
  port_is_free "$port" || fail "Port $port is in use by another program. Free it or change it in backend/.env or frontend/.env."
done

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP # the terminal was closed

step 'Backend'
(cd backend && cargo build --release)
(cd backend && exec "$BACKEND_BIN") &
pids+=($!)
until curl -fs "http://$BIND_ADDR/api/health" >/dev/null; do
  kill -0 "${pids[0]}" 2>/dev/null || fail 'Backend failed to start.'
  sleep 0.5
done

step 'Frontend'
[[ -d frontend/node_modules ]] || (cd frontend && npm ci --no-audit --no-fund)
(cd frontend && exec node "$VITE_BIN") &
pids+=($!)

printf '\n  Ieftin   http://localhost:%s\n  API      http://%s/api\n\n  Press Ctrl+C to stop everything.\n\n' \
  "$FRONTEND_PORT" "$BIND_ADDR"

while kill -0 "${pids[@]}" 2>/dev/null; do sleep 1; done
fail 'A service stopped unexpectedly; shutting down.'
