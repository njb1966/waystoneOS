#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
activation_root="${WAYSTONE_PROJECTD_DBUS_ACTIVATION_ROOT:-/tmp/waystone-projectd-dbus-activation-$$}"
create_parent="${WAYSTONE_PROJECTD_DBUS_ACTIVATION_CREATE_ROOT:-/tmp/waystone-projectd-dbus-activation-create-$$}"
service_dir="$activation_root/dbus-1/services"
service_file="$service_dir/org.waystone.Project1.service"
daemon_path="$repo_root/target/debug/waystone-projectd"

cd "$repo_root"

cargo build -p waystone-projectd

mkdir -p "$service_dir"
mkdir -p "$create_parent"

{
  printf "[D-BUS Service]\n"
  printf "Name=org.waystone.Project1\n"
  printf "Exec=%s\n" "$daemon_path"
} > "$service_file"

export WAYSTONE_PROJECTD_DBUS_ACTIVATION_CREATE_PARENT="$create_parent"
export WAYSTONE_PROJECTD_REPO_ROOT="$repo_root"

XDG_DATA_HOME="$activation_root" dbus-run-session -- bash -c '
set -eu

list_output="$(busctl --user call \
  org.waystone.Project1 \
  /org/waystone/Project \
  org.waystone.Project1 \
  ListProjects \
  s "{\"root\":\"$WAYSTONE_PROJECTD_REPO_ROOT/examples/projects\"}")"
case "$list_output" in
  *audio-capsule*minimal-capsule*ssh-capsule*) ;;
  *)
    echo "projectd D-Bus activation smoke: autostart ListProjects failed"
    echo "$list_output"
    exit 1
    ;;
esac

create_output="$(busctl --user call \
  org.waystone.Project1 \
  /org/waystone/Project \
  org.waystone.Project1 \
  CreateProject \
  s "{\"parent\":\"$WAYSTONE_PROJECTD_DBUS_ACTIVATION_CREATE_PARENT\",\"id\":\"activated-capsule\",\"name\":\"Activated Capsule\",\"type\":\"capsule\",\"content_index\":\"index.gmi\",\"language\":\"en\"}")"
case "$create_output" in
  *activated-capsule.wayproject*project_schema*) ;;
  *)
    echo "projectd D-Bus activation smoke: autostart CreateProject failed"
    echo "$create_output"
    exit 1
    ;;
esac

owner_pid="$(busctl --user call \
  org.freedesktop.DBus \
  /org/freedesktop/DBus \
  org.freedesktop.DBus \
  GetConnectionUnixProcessID \
  s org.waystone.Project1 | awk "{print \$2}")"

if [ -n "$owner_pid" ]; then
  kill -TERM "$owner_pid" 2>/dev/null || true
fi

echo "projectd D-Bus activation smoke: autostart succeeded"
'
