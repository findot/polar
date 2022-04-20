#!/usr/bin/env sh

set -eu

cd /usr/src/app

# ----------------------------------------------------------------------- UTILS

log() {
  time=$(date +"%d/%m/%Y %H:%M")
  printf "%s - %s" "$time" "$@" >&2
}

logn() {
  log "$@"
  printf "\n" >&2
}

safe_kill() {
  PID=$1
  if kill -0 "$PID" > /dev/null 2>&1; then
      kill -9 "$PID"
  fi
}

# ----------------------------------------------------------------------- LOGIC

migrate() {
  log "Starting migrations... "
  cargo install diesel_cli --no-default-features --features postgres
  if [ -f /tmp/.redo ]; then
    diesel migration redo
  else
    diesel migration run
  fi
  printf "Ok.\n" >&2
}

start() {
  log "Starting Rust server... "
  cargo build
  mkdir -p ops/dev/log
  cargo run > ops/dev/log/cargo.log 2>&1 &
  echo $! > /tmp/cargo.pid
  printf "Ok.\n" >&2
}

stop() {
  log "Terminating Rust server... "
  if [ -f /tmp/cargo.pid ]; then
    safe_kill "$(cat /tmp/cargo.pid)"
    rm /tmp/cargo.pid
  fi
  printf "Ok.\n" >&2
}

cleanup() {
  stop
  exit 0
}

# ------------------------------------------------------------------------ MAIN

migrate
start

trap cleanup INT
trap cleanup QUIT
trap cleanup TERM

log "Starting reloader... "
inotifywait -m -r -e create -e modify -e move -e delete --format "%w%f %e" src | while read -r FILE EVENT; do
  logn "Change (${EVENT}) to ${FILE} detected, reloading."
  stop
  start
done
