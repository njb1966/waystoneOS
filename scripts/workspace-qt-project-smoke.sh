#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
build_dir="${WAYSTONE_WORKSPACE_QT_BUILD_DIR:-/tmp/waystone-workspace-qt-build}"
if [ "${WAYSTONE_WORKSPACE_QT_PROJECT_SMOKE_ROOT:-}" ]; then
  smoke_root="$WAYSTONE_WORKSPACE_QT_PROJECT_SMOKE_ROOT"
else
  smoke_root="$(mktemp -d /tmp/waystone-workspace-qt-project-smoke-XXXXXX)"
fi
projects_root="$smoke_root/projects"
hosts_root="$smoke_root/hosts"
identities_root="$smoke_root/identities"
audio_root="$smoke_root/audio"
config_path="$smoke_root/workspace.ini"
project_id="workspace-smoke"
project_name="Workspace Smoke"

mkdir -p "$projects_root" "$hosts_root" "$identities_root" "$audio_root"
{
  printf "[roots]\n"
  printf "projects = %s\n" "$projects_root"
  printf "hosts = %s\n" "$hosts_root"
  printf "identities = %s\n" "$identities_root"
  printf "audio_metadata = %s\n" "$audio_root"
} > "$config_path"

cargo build -p waystone-project-cli

cmake -S "$repo_root/ui/workspace-qt" -B "$build_dir"
cmake --build "$build_dir"

set +e
output="$(QT_QPA_PLATFORM=offscreen \
  "$build_dir/waystone-workspace" \
    --repo-root "$repo_root" \
    --config "$config_path" \
    --smoke-project-create-save \
    --smoke-project-id "$project_id" \
    --smoke-project-name "$project_name" \
    --smoke-project-type capsule 2>&1)"
status=$?
set -e
if [ "$status" -ne 0 ]; then
  echo "workspace project smoke: diagnostic mode exited with $status"
  echo "$output"
  exit "$status"
fi

case "$output" in
  *"workspace project smoke: created, targeted, saved, and validated"*) ;;
  *)
    echo "workspace project smoke: diagnostic mode failed"
    echo "$output"
    exit 1
    ;;
esac

project_path="$projects_root/$project_id.wayproject"
content_path="$project_path/content/index.gmi"
if [ ! -f "$project_path/project.toml" ]; then
  echo "workspace project smoke: project manifest was not created"
  exit 1
fi

if [ ! -f "$content_path" ]; then
  echo "workspace project smoke: content index was not created"
  exit 1
fi

case "$(cat "$content_path")" in
  *"Saved from Workspace smoke."*) ;;
  *)
    echo "workspace project smoke: content index did not contain saved text"
    cat "$content_path"
    exit 1
    ;;
esac

cargo run -q -p waystone-project-cli -- validate "$project_path" >/dev/null
inspect_output="$(cargo run -q -p waystone-project-cli -- inspect --json "$project_path")"
case "$inspect_output" in
  *'"publish_targets":["export"]'*) ;;
  *)
    echo "workspace project smoke: export target was not reported by inspect"
    echo "$inspect_output"
    exit 1
    ;;
esac

echo "workspace project smoke: create/target/load/save/validate succeeded"
