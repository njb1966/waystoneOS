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
publish_project_id="workspace-publish-smoke"
publish_project_name="Workspace Publish Smoke"
recording_project_id="workspace-audio-smoke"
recording_project_name="Workspace Audio Smoke"

mkdir -p "$projects_root" "$hosts_root" "$identities_root" "$audio_root"
{
  printf "[roots]\n"
  printf "projects = %s\n" "$projects_root"
  printf "hosts = %s\n" "$hosts_root"
  printf "identities = %s\n" "$identities_root"
  printf "audio_metadata = %s\n" "$audio_root"
} > "$config_path"

cargo build -p waystone-project-cli -p waystone-publish-cli -p waystone-record-cli

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
preview_output="$(cargo run -q -p waystone-publish-cli -- \
  --dry-run \
  --project "$project_path" \
  --target export \
  --json)"
case "$preview_output" in
  *'"target":"export"'*'"method":"removable"'*) ;;
  *)
    echo "workspace project smoke: removable publish preview failed"
    echo "$preview_output"
    exit 1
    ;;
esac

set +e
publish_output="$(QT_QPA_PLATFORM=offscreen \
  "$build_dir/waystone-workspace" \
    --repo-root "$repo_root" \
    --config "$config_path" \
    --smoke-publish-target-status \
    --smoke-project-id "$publish_project_id" \
    --smoke-project-name "$publish_project_name" \
    --smoke-project-type capsule 2>&1)"
publish_status=$?
set -e
if [ "$publish_status" -ne 0 ]; then
  echo "workspace project smoke: publish diagnostic mode exited with $publish_status"
  echo "$publish_output"
  exit "$publish_status"
fi

case "$publish_output" in
  *"workspace publish smoke: target selector and status transitions succeeded"*) ;;
  *)
    echo "workspace project smoke: publish diagnostic mode failed"
    echo "$publish_output"
    exit 1
    ;;
esac

publish_project_path="$projects_root/$publish_project_id.wayproject"
publish_inspect_output="$(cargo run -q -p waystone-project-cli -- inspect --json "$publish_project_path")"
case "$publish_inspect_output" in
  *'"publish_targets":["export","backup","production"]'*) ;;
  *)
    echo "workspace project smoke: publish smoke targets were not reported by inspect"
    echo "$publish_inspect_output"
    exit 1
    ;;
esac

set +e
recording_output="$(QT_QPA_PLATFORM=offscreen \
  "$build_dir/waystone-workspace" \
    --repo-root "$repo_root" \
    --config "$config_path" \
    --smoke-recording-attach \
    --smoke-project-id "$recording_project_id" \
    --smoke-project-name "$recording_project_name" \
    --smoke-project-type audio-series 2>&1)"
recording_status=$?
set -e
if [ "$recording_status" -ne 0 ]; then
  echo "workspace project smoke: recording diagnostic mode exited with $recording_status"
  echo "$recording_output"
  exit "$recording_status"
fi

case "$recording_output" in
  *"workspace recording smoke: attachment and feed-entry controls succeeded"*) ;;
  *)
    echo "workspace project smoke: recording diagnostic mode failed"
    echo "$recording_output"
    exit 1
    ;;
esac

recording_project_path="$projects_root/$recording_project_id.wayproject"
recording_metadata_path="$recording_project_path/audio/metadata/field-note.toml"
feed_entry_path="$recording_project_path/feeds/entries/field-note.toml"
if [ ! -f "$recording_metadata_path" ]; then
  echo "workspace project smoke: recording metadata sidecar was not created"
  exit 1
fi
if [ ! -f "$feed_entry_path" ]; then
  echo "workspace project smoke: feed-entry sidecar was not created"
  exit 1
fi

recording_inspect_output="$(cargo run -q -p waystone-record-cli -- inspect --json "$recording_metadata_path")"
case "$recording_inspect_output" in
  *'"id":"field-note"'*'"published":"audio/published/field-note.opus"'*) ;;
  *)
    echo "workspace project smoke: attached recording was not inspectable"
    echo "$recording_inspect_output"
    exit 1
    ;;
esac
recording_publication_validation="$(cargo run -q -p waystone-record-cli -- validate-publication --json "$recording_project_path" field-note)"
case "$recording_publication_validation" in
  *'"valid":true'*) ;;
  *)
    echo "workspace project smoke: publication validation failed"
    echo "$recording_publication_validation"
    exit 1
    ;;
esac
recording_feed_validation="$(cargo run -q -p waystone-record-cli -- validate-feed-entry --json "$recording_project_path" field-note)"
case "$recording_feed_validation" in
  *'"valid":true'*) ;;
  *)
    echo "workspace project smoke: feed-entry validation failed"
    echo "$recording_feed_validation"
    exit 1
    ;;
esac

echo "workspace project smoke: create/target/load/save/validate/preview/status/recording succeeded"
