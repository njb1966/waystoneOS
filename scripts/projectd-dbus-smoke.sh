#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
daemon_log="${WAYSTONE_PROJECTD_DBUS_LOG:-/tmp/waystone-projectd-dbus-smoke.log}"
create_parent="${WAYSTONE_PROJECTD_DBUS_CREATE_ROOT:-/tmp/waystone-projectd-dbus-smoke-$$}"
cd "$repo_root"

mkdir -p "$create_parent"
export WAYSTONE_PROJECTD_DBUS_CREATE_PARENT="$create_parent"

cargo build -p waystone-projectd

dbus-run-session -- bash -c '
set -eu

target/debug/waystone-projectd > "$0" 2>&1 &
daemon_pid=$!

cleanup() {
  kill -TERM "$daemon_pid" 2>/dev/null || true
  wait "$daemon_pid" 2>/dev/null || true
}
trap cleanup EXIT

ready=0
for _ in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 19 20; do
  if busctl --user status org.waystone.Project1 >/dev/null 2>&1; then
    ready=1
    break
  fi
  sleep 0.1
done

if [ "$ready" -ne 1 ]; then
  echo "projectd D-Bus smoke: daemon did not claim org.waystone.Project1"
  cat "$0"
  exit 1
fi

list_output="$(busctl --user call \
  org.waystone.Project1 \
  /org/waystone/Project \
  org.waystone.Project1 \
  ListProjects \
  s "{\"root\":\"examples/projects\"}")"
case "$list_output" in
  *audio-capsule*minimal-capsule*ssh-capsule*) ;;
  *)
    echo "projectd D-Bus smoke: ListProjects did not report expected projects"
    echo "$list_output"
    exit 1
    ;;
esac

inspect_output="$(busctl --user call \
  org.waystone.Project1 \
  /org/waystone/Project \
  org.waystone.Project1 \
  InspectProject \
  s "{\"path\":\"examples/projects/minimal-capsule.wayproject\"}")"
case "$inspect_output" in
  *minimal-capsule*Minimal\ Capsule*) ;;
  *)
    echo "projectd D-Bus smoke: InspectProject did not report expected identity"
    echo "$inspect_output"
    exit 1
    ;;
esac

validate_output="$(busctl --user call \
  org.waystone.Project1 \
  /org/waystone/Project \
  org.waystone.Project1 \
  ValidateProject \
  s "{\"path\":\"tests/fixtures/projects/invalid-missing-index.wayproject\"}")"
case "$validate_output" in
  *missing_content_index*) ;;
  *)
    echo "projectd D-Bus smoke: ValidateProject did not report invalid fixture"
    echo "$validate_output"
    exit 1
    ;;
esac

bad_request_output="$(busctl --user call \
  org.waystone.Project1 \
  /org/waystone/Project \
  org.waystone.Project1 \
  ListProjects \
  s "{bad-json")"
case "$bad_request_output" in
  *invalid_request*) ;;
  *)
    echo "projectd D-Bus smoke: invalid request was not reported"
    echo "$bad_request_output"
    exit 1
    ;;
esac

create_output="$(busctl --user call \
  org.waystone.Project1 \
  /org/waystone/Project \
  org.waystone.Project1 \
  CreateProject \
  s "{\"parent\":\"$WAYSTONE_PROJECTD_DBUS_CREATE_PARENT\",\"id\":\"dbus-capsule\",\"name\":\"D-Bus Capsule\",\"type\":\"capsule\",\"content_index\":\"index.gmi\",\"language\":\"en\"}")"
case "$create_output" in
  *dbus-capsule.wayproject*project_schema*) ;;
  *)
    echo "projectd D-Bus smoke: CreateProject did not report created project"
    echo "$create_output"
    exit 1
    ;;
esac

created_project="$WAYSTONE_PROJECTD_DBUS_CREATE_PARENT/dbus-capsule.wayproject"
created_inspect_output="$(busctl --user call \
  org.waystone.Project1 \
  /org/waystone/Project \
  org.waystone.Project1 \
  InspectProject \
  s "{\"path\":\"$created_project\"}")"
case "$created_inspect_output" in
  *dbus-capsule*D-Bus\ Capsule*) ;;
  *)
    echo "projectd D-Bus smoke: created project could not be inspected"
    echo "$created_inspect_output"
    exit 1
    ;;
esac

echo "projectd D-Bus smoke: project adapter methods succeeded"
' "$daemon_log"
