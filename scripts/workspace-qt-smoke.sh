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
  timeout 5s "$build_dir/waystone-workspace" --repo-root "$repo_root"
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
  if [ "$config_status" -eq 124 ]; then
    echo "waystone-workspace smoke: default and config startup succeeded"
    exit 0
  fi

  echo "waystone-workspace smoke: config startup failed with exit code $config_status"
  exit "$config_status"
fi

if [ "$status" -eq 0 ]; then
  echo "waystone-workspace smoke: app exited before timeout"
  exit 1
fi

echo "waystone-workspace smoke: failed with exit code $status"
exit "$status"
