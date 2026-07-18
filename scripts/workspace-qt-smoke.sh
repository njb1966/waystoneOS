#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
build_dir="${WAYSTONE_WORKSPACE_QT_BUILD_DIR:-/tmp/waystone-workspace-qt-build}"

cargo build \
  -p waystone-project-cli \
  -p waystone-publish-cli \
  -p waystone-host-cli \
  -p waystone-identity-cli \
  -p waystone-record-cli \
  -p waystone-listen-cli

cmake -S "$repo_root/ui/workspace-qt" -B "$build_dir"
cmake --build "$build_dir"

set +e
QT_QPA_PLATFORM=offscreen \
  timeout 5s "$build_dir/waystone-workspace" \
    --repo-root "$repo_root" \
    --no-user-config
status=$?
set -e

if [ "$status" -eq 124 ]; then
  set +e
  QT_QPA_PLATFORM=offscreen \
    timeout 5s "$build_dir/waystone-workspace" \
      --repo-root "$repo_root" \
      --config "$repo_root/ui/workspace-qt/workspace.example.ini"
  config_status=$?
  set -e
  if [ "$config_status" -ne 124 ]; then
    echo "waystone-workspace smoke: config startup failed with exit code $config_status"
    exit "$config_status"
  fi

  user_config_home="${WAYSTONE_WORKSPACE_QT_CONFIG_HOME:-/tmp/waystone-workspace-qt-config}"
  mkdir -p "$user_config_home/WaystoneOS/Waystone Workspace"
  cp "$repo_root/ui/workspace-qt/workspace.example.ini" \
    "$user_config_home/WaystoneOS/Waystone Workspace/workspace.ini"

  set +e
  XDG_CONFIG_HOME="$user_config_home" \
    QT_QPA_PLATFORM=offscreen \
    timeout 5s "$build_dir/waystone-workspace" --repo-root "$repo_root"
  user_config_status=$?
  set -e
  if [ "$user_config_status" -eq 124 ]; then
    echo "waystone-workspace smoke: default, explicit config, and user config startup succeeded"
    exit 0
  fi

  echo "waystone-workspace smoke: user config startup failed with exit code $user_config_status"
  exit "$user_config_status"
fi

if [ "$status" -eq 0 ]; then
  echo "waystone-workspace smoke: app exited before timeout"
  exit 1
fi

echo "waystone-workspace smoke: failed with exit code $status"
exit "$status"
