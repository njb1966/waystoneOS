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
  echo "waystone-workspace smoke: startup succeeded and event loop remained active"
  exit 0
fi

if [ "$status" -eq 0 ]; then
  echo "waystone-workspace smoke: app exited before timeout"
  exit 1
fi

echo "waystone-workspace smoke: failed with exit code $status"
exit "$status"
