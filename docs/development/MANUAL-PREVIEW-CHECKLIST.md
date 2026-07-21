# Manual Preview Checklist

Status: Current as of 2026-07-21

This checklist is for manually exercising the current repo-local WaystoneOS
development preview before any installed session work.

Start with the shorter project-owner flow in
[WORKSPACE-PREVIEW-GUIDE.md](WORKSPACE-PREVIEW-GUIDE.md). Use this checklist
for setup commands and deeper optional checks.

It uses `scripts/run-dev-session.sh`, which builds the Rust CLIs and Qt
Workspace, then launches through `session/waystone-session`. It does not
install files into `/usr`, register with a display manager, install D-Bus
activation files, or install systemd user units.

## Preconditions

Expected Debian packages:

```text
cmake
g++
qt6-base-dev
```

Expected external tools for the current audio preview path:

```text
ffmpeg
```

## Fast Sanity Check

Run from the repository root:

```bash
QT_QPA_PLATFORM=offscreen scripts/run-dev-session.sh --check-roots --no-user-config
```

Expected result:

```text
config source: defaults
roots: ok
```

## Isolated Manual Workspace

Create temporary roots for manual testing:

```bash
preview_root="$(mktemp -d /tmp/waystone-manual-preview-XXXXXX)"
mkdir -p "$preview_root/projects" "$preview_root/hosts" "$preview_root/identities" "$preview_root/audio"
```

Write a temporary Workspace config:

```bash
preview_config="$preview_root/workspace.ini"
printf "[roots]\n" > "$preview_config"
printf "projects = %s\n" "$preview_root/projects" >> "$preview_config"
printf "hosts = %s\n" "$preview_root/hosts" >> "$preview_config"
printf "identities = %s\n" "$preview_root/identities" >> "$preview_config"
printf "audio_metadata = %s\n" "$preview_root/audio" >> "$preview_config"
```

Confirm the temporary roots are valid:

```bash
QT_QPA_PLATFORM=offscreen scripts/run-dev-session.sh --config "$preview_config" --check-roots
```

Expected result:

```text
config source: explicit
roots: ok
```

Launch the preview:

```bash
scripts/run-dev-session.sh --config "$preview_config"
```

## Golden Path Check

This is the main manual pass for the current preview:

Explore:

- The application opens as Waystone Workspace, not as an installed OS session.
- The four workspace selectors are visible: Explore, Create, Publish, Operate.
- The active roots match the temporary config paths.

Create:

- Create a small `capsule` project.
- Confirm the new project appears in the project list.
- In `Write`, edit the Gemtext index and save.
- Confirm the local preview and link validation update from the saved text.
- In `Files`, confirm the content-file list shows project-local files.

Publish:

- Select the project and `export` target.
- In `Preview`, confirm the dry-run, validation, transfer-intent, and removable
  readiness reports are readable.
- Confirm the reports are preview/read-only status and do not transfer files.

Stop here for the basic pass.

## Optional Deeper Checks

Create audio path:

- Create an `audio-series` project.
- Confirm audio and feed scaffold defaults exist.
- Use the explicit-input capture controls only if `ffmpeg` is available.
- Export an Opus publication copy from a project-local master.
- Attach or update recording metadata.
- Prepare or update a feed entry and generate the local Atom feed.

Publish:

- Confirm dry-run preview shows planned local file actions.
- Confirm publication readiness reports ready or blocked state without
  transferring files.
- Confirm transfer-intent and removable execution readiness are read-only.
- Confirm saved planned previews and completed-history list/detail views are
  local project records.

Operate:

- Confirm host and identity panes load from the configured roots.
- Empty temporary roots should be understandable as empty state, not failure.

## Expected Deferrals

Do not expect these to work yet:

- selecting WaystoneOS from a display manager
- booting a WaystoneOS image
- installed D-Bus activation
- installed systemd user units
- SSH-family publication transfer
- remote deletion execution
- credential unlock
- remote verification
- audio device enumeration
- audio playback
- Browser, Helm, or Comm embedding

## Record Findings

For each manual pass, record:

- date
- command used
- config path used
- whether the window opened
- what workflow was tested
- any visible UI confusion
- any command output or terminal error
- whether repository examples or only temporary roots were used

Current recorded findings live in
[MANUAL-PREVIEW-FINDINGS.md](MANUAL-PREVIEW-FINDINGS.md).
