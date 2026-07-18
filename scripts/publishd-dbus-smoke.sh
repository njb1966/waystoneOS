#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
daemon_log="${WAYSTONE_PUBLISHD_DBUS_LOG:-/tmp/waystone-publishd-dbus-smoke.log}"
cd "$repo_root"

cargo build -p waystone-publishd

missing_bus="unix:path=/tmp/waystone-publishd-missing-bus-$$"
set +e
missing_bus_output="$(DBUS_SESSION_BUS_ADDRESS="$missing_bus" \
  target/debug/waystone-publishd 2>&1)"
missing_bus_status=$?
set -e
if [ "$missing_bus_status" -eq 0 ]; then
  echo "publishd D-Bus smoke: daemon started without an available session bus"
  echo "$missing_bus_output"
  exit 1
fi

case "$missing_bus_output" in
  *waystone-publishd:*) ;;
  *)
    echo "publishd D-Bus smoke: unavailable bus failure was not reported clearly"
    echo "$missing_bus_output"
    exit 1
    ;;
esac

dbus-run-session -- bash -c '
set -eu

target/debug/waystone-publishd > "$0" 2>&1 &
daemon_pid=$!

cleanup() {
  kill -TERM "$daemon_pid" 2>/dev/null || true
  wait "$daemon_pid" 2>/dev/null || true
}
trap cleanup EXIT

ready=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
  if busctl --user status org.waystone.Publish1 >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 0.1
done

if [ "$ready" -ne 1 ]; then
  echo "publishd D-Bus smoke: daemon did not claim org.waystone.Publish1"
  cat "$0"
  exit 1
fi

duplicate_log="$0.duplicate"
set +e
timeout 3s target/debug/waystone-publishd > "$duplicate_log" 2>&1
duplicate_status=$?
set -e
if [ "$duplicate_status" -eq 0 ] || [ "$duplicate_status" -eq 124 ]; then
  echo "publishd D-Bus smoke: duplicate daemon did not fail quickly"
  cat "$duplicate_log"
  exit 1
fi

case "$(cat "$duplicate_log")" in
  *waystone-publishd:*) ;;
  *)
    echo "publishd D-Bus smoke: duplicate ownership failure was not reported clearly"
    cat "$duplicate_log"
    exit 1
    ;;
esac

preview_output="$(busctl --user call \
  org.waystone.Publish1 \
  /org/waystone/Publish \
  org.waystone.Publish1 \
  PreviewPublication \
  s "{\"project_path\":\"examples/projects/ssh-capsule.wayproject\",\"target\":\"production\",\"hosts_root\":\"examples/connections/hosts\",\"identities_root\":\"examples/connections/identities\"}")"
for expected in ssh-capsule production rsync offgridholdout nick-pub content/index.gmi; do
  case "$preview_output" in
    *"$expected"*) ;;
    *)
      echo "publishd D-Bus smoke: PreviewPublication did not report expected plan"
      echo "$preview_output"
      exit 1
      ;;
  esac
done

history_output="$(busctl --user call \
  org.waystone.Publish1 \
  /org/waystone/Publish \
  org.waystone.Publish1 \
  BuildPlannedHistory \
  s "{\"project_path\":\"examples/projects/ssh-capsule.wayproject\",\"target\":\"production\",\"hosts_root\":\"examples/connections/hosts\",\"identities_root\":\"examples/connections/identities\",\"date\":\"2026-07-18T00:00:00Z\"}")"
for expected in planned not-run planned-upload content/index.gmi; do
  case "$history_output" in
    *"$expected"*) ;;
    *)
      echo "publishd D-Bus smoke: BuildPlannedHistory did not report expected planned record"
      echo "$history_output"
      exit 1
      ;;
  esac
done

bad_request_output="$(busctl --user call \
  org.waystone.Publish1 \
  /org/waystone/Publish \
  org.waystone.Publish1 \
  PreviewPublication \
  s "{bad-json")"
case "$bad_request_output" in
  *invalid_request*) ;;
  *)
    echo "publishd D-Bus smoke: invalid request was not reported"
    echo "$bad_request_output"
    exit 1
    ;;
esac

echo "publishd D-Bus smoke: adapter methods succeeded"
' "$daemon_log"
