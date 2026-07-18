#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
activation_root="${WAYSTONE_AUDIOD_DBUS_ACTIVATION_ROOT:-/tmp/waystone-audiod-dbus-activation-$$}"
service_dir="$activation_root/dbus-1/services"
service_file="$service_dir/org.waystone.Audio1.service"
daemon_path="$repo_root/target/debug/waystone-audiod"

cd "$repo_root"

cargo build -p waystone-audiod

mkdir -p "$service_dir"

{
  printf "[D-BUS Service]\n"
  printf "Name=org.waystone.Audio1\n"
  printf "Exec=%s\n" "$daemon_path"
} > "$service_file"

export WAYSTONE_AUDIOD_REPO_ROOT="$repo_root"

XDG_DATA_HOME="$activation_root" dbus-run-session -- bash -c '
set -eu

list_output="$(busctl --user call \
  org.waystone.Audio1 \
  /org/waystone/Audio \
  org.waystone.Audio1 \
  ListRecordings \
  s "{\"root\":\"$WAYSTONE_AUDIOD_REPO_ROOT/examples/projects/audio-capsule.wayproject/audio/metadata\"}")"
case "$list_output" in
  *field-note*) ;;
  *)
    echo "audiod D-Bus activation smoke: autostart ListRecordings failed"
    echo "$list_output"
    exit 1
    ;;
esac

owner_pid="$(busctl --user call \
  org.freedesktop.DBus \
  /org/freedesktop/DBus \
  org.freedesktop.DBus \
  GetConnectionUnixProcessID \
  s org.waystone.Audio1 | awk "{print \$2}")"

if [ -n "$owner_pid" ]; then
  kill -TERM "$owner_pid" 2>/dev/null || true
fi

echo "audiod D-Bus activation smoke: autostart succeeded"
'
