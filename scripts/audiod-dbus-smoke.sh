#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
daemon_log="${WAYSTONE_AUDIOD_DBUS_LOG:-/tmp/waystone-audiod-dbus-smoke.log}"
cd "$repo_root"

cargo build -p waystone-audiod

missing_bus="unix:path=/tmp/waystone-audiod-missing-bus-$$"
set +e
missing_bus_output="$(DBUS_SESSION_BUS_ADDRESS="$missing_bus" \
  target/debug/waystone-audiod 2>&1)"
missing_bus_status=$?
set -e
if [ "$missing_bus_status" -eq 0 ]; then
  echo "audiod D-Bus smoke: daemon started without an available session bus"
  echo "$missing_bus_output"
  exit 1
fi

case "$missing_bus_output" in
  *waystone-audiod:*) ;;
  *)
    echo "audiod D-Bus smoke: unavailable bus failure was not reported clearly"
    echo "$missing_bus_output"
    exit 1
    ;;
esac

dbus-run-session -- bash -c '
set -eu

target/debug/waystone-audiod > "$0" 2>&1 &
daemon_pid=$!

cleanup() {
  kill -TERM "$daemon_pid" 2>/dev/null || true
  wait "$daemon_pid" 2>/dev/null || true
}
trap cleanup EXIT

ready=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
  if busctl --user status org.waystone.Audio1 >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 0.1
done

if [ "$ready" -ne 1 ]; then
  echo "audiod D-Bus smoke: daemon did not claim org.waystone.Audio1"
  cat "$0"
  exit 1
fi

duplicate_log="$0.duplicate"
set +e
timeout 3s target/debug/waystone-audiod > "$duplicate_log" 2>&1
duplicate_status=$?
set -e
if [ "$duplicate_status" -eq 0 ] || [ "$duplicate_status" -eq 124 ]; then
  echo "audiod D-Bus smoke: duplicate daemon did not fail quickly"
  cat "$duplicate_log"
  exit 1
fi

case "$(cat "$duplicate_log")" in
  *waystone-audiod:*) ;;
  *)
    echo "audiod D-Bus smoke: duplicate ownership failure was not reported clearly"
    cat "$duplicate_log"
    exit 1
    ;;
esac

list_output="$(busctl --user call \
  org.waystone.Audio1 \
  /org/waystone/Audio \
  org.waystone.Audio1 \
  ListRecordings \
  s "{\"root\":\"examples/projects/audio-capsule.wayproject/audio/metadata\"}")"
case "$list_output" in
  *field-note*) ;;
  *)
    echo "audiod D-Bus smoke: ListRecordings did not report expected recording"
    echo "$list_output"
    exit 1
    ;;
esac

inspect_output="$(busctl --user call \
  org.waystone.Audio1 \
  /org/waystone/Audio \
  org.waystone.Audio1 \
  InspectRecording \
  s "{\"path\":\"examples/projects/audio-capsule.wayproject/audio/metadata/field-note.toml\"}")"
case "$inspect_output" in
  *field-note.flac*field-note.opus*) ;;
  *)
    echo "audiod D-Bus smoke: InspectRecording did not report expected recording"
    echo "$inspect_output"
    exit 1
    ;;
esac

validate_output="$(busctl --user call \
  org.waystone.Audio1 \
  /org/waystone/Audio \
  org.waystone.Audio1 \
  ValidateRecording \
  s "{\"path\":\"tests/fixtures/audio/invalid-path/field-note.toml\"}")"
case "$validate_output" in
  *invalid_audio_path*) ;;
  *)
    echo "audiod D-Bus smoke: ValidateRecording did not report invalid fixture"
    echo "$validate_output"
    exit 1
    ;;
esac

bad_request_output="$(busctl --user call \
  org.waystone.Audio1 \
  /org/waystone/Audio \
  org.waystone.Audio1 \
  ListRecordings \
  s "{bad-json")"
case "$bad_request_output" in
  *invalid_request*) ;;
  *)
    echo "audiod D-Bus smoke: invalid request was not reported"
    echo "$bad_request_output"
    exit 1
    ;;
esac

echo "audiod D-Bus smoke: adapter methods succeeded"
' "$daemon_log"
