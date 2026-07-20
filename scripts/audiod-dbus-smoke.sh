#!/usr/bin/env bash
set -eu

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
daemon_log="${WAYSTONE_AUDIOD_DBUS_LOG:-/tmp/waystone-audiod-dbus-smoke.log}"
cd "$repo_root"

cargo build -p waystone-audiod

if ! command -v ffmpeg >/dev/null 2>&1; then
  echo "audiod D-Bus smoke: ffmpeg is required for mutating audio method smoke"
  exit 1
fi
if ! ffmpeg -hide_banner -encoders 2>/dev/null | grep -q libopus; then
  echo "audiod D-Bus smoke: ffmpeg libopus encoder is required for mutating audio method smoke"
  exit 1
fi

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

temp_project="$(mktemp -d /tmp/waystone-audiod-dbus-project-XXXXXX)"
mkdir -p \
  "$temp_project/audio/masters" \
  "$temp_project/audio/published" \
  "$temp_project/audio/metadata" \
  "$temp_project/feeds/entries"

capture_output="$(busctl --user call \
  org.waystone.Audio1 \
  /org/waystone/Audio \
  org.waystone.Audio1 \
  CaptureRecording \
  s "{\"project_root\":\"$temp_project\",\"masters_root\":\"audio/masters\",\"master\":\"audio/masters/field-note.wav\",\"duration_seconds\":1,\"input_format\":\"lavfi\",\"input\":\"anullsrc=r=48000:cl=mono\"}")"
case "$capture_output" in
  *audio/masters/field-note.wav*ffmpeg*|*ffmpeg*audio/masters/field-note.wav*) ;;
  *)
    echo "audiod D-Bus smoke: CaptureRecording did not report captured master"
    echo "$capture_output"
    exit 1
    ;;
esac
if [ ! -f "$temp_project/audio/masters/field-note.wav" ]; then
  echo "audiod D-Bus smoke: CaptureRecording did not write master"
  exit 1
fi

export_output="$(busctl --user call \
  org.waystone.Audio1 \
  /org/waystone/Audio \
  org.waystone.Audio1 \
  ExportOpus \
  s "{\"project_root\":\"$temp_project\",\"master\":\"audio/masters/field-note.wav\",\"published\":\"audio/published/field-note.opus\",\"preset\":\"voice-standard\"}")"
case "$export_output" in
  *audio/published/field-note.opus*ffmpeg*|*ffmpeg*audio/published/field-note.opus*) ;;
  *)
    echo "audiod D-Bus smoke: ExportOpus did not report publication copy"
    echo "$export_output"
    exit 1
    ;;
esac
if [ ! -f "$temp_project/audio/published/field-note.opus" ]; then
  echo "audiod D-Bus smoke: ExportOpus did not write publication copy"
  exit 1
fi

attach_output="$(busctl --user call \
  org.waystone.Audio1 \
  /org/waystone/Audio \
  org.waystone.Audio1 \
  AttachRecording \
  s "{\"project_root\":\"$temp_project\",\"metadata_root\":\"audio/metadata\",\"id\":\"field-note\",\"title\":\"Field Note\",\"master\":\"audio/masters/field-note.wav\",\"published\":\"audio/published/field-note.opus\",\"feed\":\"feeds/feed.xml\",\"entry_id\":\"tag:example.invalid,2026:field-note\",\"mime_type\":\"audio/ogg; codecs=opus\"}")"
case "$attach_output" in
  *audio/metadata/field-note.toml*field-note*) ;;
  *)
    echo "audiod D-Bus smoke: AttachRecording did not report metadata sidecar"
    echo "$attach_output"
    exit 1
    ;;
esac
metadata_path="$temp_project/audio/metadata/field-note.toml"
if [ ! -f "$metadata_path" ]; then
  echo "audiod D-Bus smoke: AttachRecording did not write metadata sidecar"
  exit 1
fi

update_output="$(busctl --user call \
  org.waystone.Audio1 \
  /org/waystone/Audio \
  org.waystone.Audio1 \
  UpdateRecording \
  s "{\"project_root\":\"$temp_project\",\"recording_metadata_path\":\"$metadata_path\",\"title\":\"Field Note Revised\",\"master\":\"audio/masters/field-note.wav\",\"published\":\"audio/published/field-note.opus\",\"feed\":\"feeds/feed.xml\",\"entry_id\":\"tag:example.invalid,2026:field-note-revised\",\"mime_type\":\"audio/ogg; codecs=opus\"}")"
case "$update_output" in
  *"Field Note Revised"*field-note-revised*|*field-note-revised*"Field Note Revised"*) ;;
  *)
    echo "audiod D-Bus smoke: UpdateRecording did not report revised metadata"
    echo "$update_output"
    exit 1
    ;;
esac

prepare_output="$(busctl --user call \
  org.waystone.Audio1 \
  /org/waystone/Audio \
  org.waystone.Audio1 \
  PrepareFeedEntry \
  s "{\"project_root\":\"$temp_project\",\"recording_metadata_path\":\"$metadata_path\",\"updated\":\"2026-07-20T00:00:00Z\",\"summary\":\"D-Bus feed entry smoke\"}")"
case "$prepare_output" in
  *feeds/entries/field-note.toml*field-note-revised*|*field-note-revised*feeds/entries/field-note.toml*) ;;
  *)
    echo "audiod D-Bus smoke: PrepareFeedEntry did not report feed-entry sidecar"
    echo "$prepare_output"
    exit 1
    ;;
esac
feed_entry_path="$temp_project/feeds/entries/field-note.toml"
if [ ! -f "$feed_entry_path" ]; then
  echo "audiod D-Bus smoke: PrepareFeedEntry did not write feed-entry sidecar"
  exit 1
fi

publication_validation_output="$(busctl --user call \
  org.waystone.Audio1 \
  /org/waystone/Audio \
  org.waystone.Audio1 \
  ValidatePublication \
  s "{\"project_root\":\"$temp_project\",\"recording_metadata_path\":\"$metadata_path\"}")"
case "$publication_validation_output" in
  *valid*true*) ;;
  *)
    echo "audiod D-Bus smoke: ValidatePublication did not report valid handoff"
    echo "$publication_validation_output"
    exit 1
    ;;
esac

feed_validation_output="$(busctl --user call \
  org.waystone.Audio1 \
  /org/waystone/Audio \
  org.waystone.Audio1 \
  ValidateFeedEntry \
  s "{\"project_root\":\"$temp_project\",\"feed_entry_path\":\"$feed_entry_path\"}")"
case "$feed_validation_output" in
  *valid*true*) ;;
  *)
    echo "audiod D-Bus smoke: ValidateFeedEntry did not report valid handoff"
    echo "$feed_validation_output"
    exit 1
    ;;
esac

update_feed_output="$(busctl --user call \
  org.waystone.Audio1 \
  /org/waystone/Audio \
  org.waystone.Audio1 \
  UpdateFeedEntry \
  s "{\"project_root\":\"$temp_project\",\"recording_metadata_path\":\"$metadata_path\",\"updated\":\"2026-07-20T01:00:00Z\",\"summary\":\"D-Bus feed entry update smoke\"}")"
case "$update_feed_output" in
  *2026-07-20T01:00:00Z*field-note-revised*|*field-note-revised*2026-07-20T01:00:00Z*) ;;
  *)
    echo "audiod D-Bus smoke: UpdateFeedEntry did not report updated sidecar"
    echo "$update_feed_output"
    exit 1
    ;;
esac

generate_feed_output="$(busctl --user call \
  org.waystone.Audio1 \
  /org/waystone/Audio \
  org.waystone.Audio1 \
  GenerateFeed \
  s "{\"project_root\":\"$temp_project\",\"feed_path\":\"feeds/feed.xml\",\"title\":\"D-Bus Smoke Feed\"}")"
case "$generate_feed_output" in
  *feeds/feed.xml*entries*1*|*entries*1*feeds/feed.xml*) ;;
  *)
    echo "audiod D-Bus smoke: GenerateFeed did not report generated feed"
    echo "$generate_feed_output"
    exit 1
    ;;
esac
if [ ! -f "$temp_project/feeds/feed.xml" ]; then
  echo "audiod D-Bus smoke: GenerateFeed did not write feed XML"
  exit 1
fi

echo "audiod D-Bus smoke: adapter methods succeeded"
' "$daemon_log"
