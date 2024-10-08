#!/usr/bin/env sh

set -eu

cd /usr/src/app

# ----------------------------------------------------------------------- UTILS

log() {
  time=$(date +"%d/%m/%Y %H:%M")
  printf "%s - %s" "$time" "$@" >&2
}

logn() {
  log "$@\n"
}

safe_kill() {
  PID=$1
  if kill -0 "$PID" > /dev/null 2>&1; then
      kill -9 "$PID"
  fi
}

# ----------------------------------------------------------------------- LOGIC

setup() {
    chown -R polar:polar /usr/local/cargo/registry
}

migrate() {
  logn "Starting migrations... "
  cargo run -- -C ./ops/dev/polar.toml migrate
}

start() {
  logn "Starting Rust server... "
  cargo build
  mkdir -p ops/dev/log
  cargo run -- -C /etc/polar/polar.toml serve &
  echo $! > /tmp/cargo.pid
}

stop() {
  logn "Terminating Rust server... "
  if [ -f /tmp/cargo.pid ]; then
    safe_kill "$(cat /tmp/cargo.pid)"
    rm /tmp/cargo.pid
  fi
}

cleanup() {
  logn "Termination signal received, exiting..."
  stop
  exit 0
}

# ------------------------------------------------------------------------ MAIN

main() {
    if [ "$(id -u)" -eq 0 ]; then
        setup
        setpriv --reuid polar --regid polar --init-groups $0
    else
        migrate
        start

        trap cleanup INT
        trap cleanup QUIT
        trap cleanup TERM

        logn "Starting reloader... "
        inotifywait -m -r -e create -e modify -e move -e delete --format "%w%f %e" src | while read -r FILE EVENT; do
            logn "Change (${EVENT}) to ${FILE} detected, reloading."
            stop
            start
        done
    fi
}

main
