# Manual Preview Findings

Status: Current as of 2026-07-21

This file records manual-preview findings for the repo-local development
session path described in [MANUAL-PREVIEW-CHECKLIST.md](MANUAL-PREVIEW-CHECKLIST.md).

## 2026-07-21 Agent-Observable Pass

Scope:

- repo-local only
- no installation into `/usr`, `$XDG_DATA_HOME`, D-Bus service directories, or
  systemd user directories
- temporary roots under `/tmp`
- terminal-observable checks and headless Qt diagnostics
- no human-visible UI inspection

Commands run:

```bash
QT_QPA_PLATFORM=offscreen scripts/run-dev-session.sh --check-roots --no-user-config
```

Result:

```text
config source: defaults
roots: ok
```

Command:

```bash
QT_QPA_PLATFORM=offscreen scripts/run-dev-session.sh --config "$preview_config" --check-roots
```

Result:

```text
config source: explicit
roots: ok
```

Command:

```bash
QT_QPA_PLATFORM=offscreen timeout 5s scripts/run-dev-session.sh --config "$preview_config"
```

Result:

- exited by timeout with status `124`
- treated as startup success for the headless probe because the Qt event loop
  remained active until the timeout

Command:

```bash
scripts/workspace-qt-project-smoke.sh
```

Result:

```text
workspace project smoke: create/target/load/save/validate/preview/status/recording/feed succeeded
```

## Findings

- `scripts/run-dev-session.sh` successfully builds the Rust CLIs and Qt
  Workspace, then routes execution through `session/waystone-session`.
- Default-root diagnostics pass through the dev-run path.
- Explicit temporary-root diagnostics pass through the dev-run path.
- Offscreen startup reaches the Qt event loop.
- The existing focused Qt smoke covers project creation, content editing,
  validation, publish preview/status, recording capture/export/metadata,
  feed-entry operations, and feed generation against temporary roots.
- No repository examples were modified by the checklist commands.
- No files were installed outside the repository.

## Environment Notes

An attempted `xvfb-run` launch did not start the UI in this agent environment.
Qt failed to initialize Wayland/XCB, and the XCB error mentioned the Qt 6
`xcb-cursor0` or `libxcb-cursor0` dependency. This is recorded as an
environment limitation for the headless/virtual-display probe, not as a
confirmed WaystoneOS application failure.

## Still Needs Human Observation

A project owner should still run the checklist in a visible desktop session and
record:

- whether the window opens normally
- whether the four workspace selectors are visible and understandable
- whether the active roots shown in Explore match the temporary config
- whether Create, Publish, and Operate panes are understandable without reading
  code
- any visual spacing, focus, contrast, wording, or workflow confusion

## Current Judgment

The repo-local session path is ready for a visible manual preview pass. It is
not ready to be treated as an installed OS session, display-manager entry,
bootable image, or installed D-Bus activation path.

## 2026-07-21 Project-Owner Visible Pass

Observed issue:

- The Create pane stacked project controls, Gemtext authoring, file detail,
  target setup, and recording controls in one crowded vertical surface.
- The Gemtext editor and preview collapsed to a very small height and were not
  practically usable.
- The audio section appeared without a clear workflow and visually collided
  with the authoring surface.

Resolution:

- The Create pane now separates work into `Write`, `Files`, and `Recordings`
  tabs.
- The Write tab gives the Gemtext editor and preview stable vertical space.
- The Files tab holds content-file filtering, file list, and file detail.
- The Recordings tab is scrollable and contains the audio capture/export,
  metadata, feed-entry, and recording list controls.

Verification:

- `cmake --build /tmp/waystone-workspace-qt-build`
- `scripts/workspace-qt-project-smoke.sh`
- `scripts/workspace-qt-smoke.sh`
- `QT_QPA_PLATFORM=offscreen scripts/run-dev-session.sh --check-roots --no-user-config`
- `git diff --check`

Second observed issue:

- The Publish pane stacked dry-run, validation, transfer-intent, removable
  readiness, feed diagnostic, saved-preview, completed-history, and planned
  history reports in one crowded vertical surface.
- Those report areas collapsed into one-line bands in the visible preview.

Resolution:

- The Publish pane now separates reports into `Preview`, `Feed`, `Planned
  History`, and `Completed History` tabs.
- The project list and target overview remain visible above the tabs.
- Existing preview, validation, transfer-intent, removable readiness,
  feed-diagnostic, saved-preview, and completed-history controls keep their
  existing object names and CLI adapter behavior.

Verification:

- `cmake --build /tmp/waystone-workspace-qt-build`
- `scripts/workspace-qt-project-smoke.sh`
- `scripts/workspace-qt-smoke.sh`
- `QT_QPA_PLATFORM=offscreen scripts/run-dev-session.sh --check-roots --no-user-config`
- `git diff --check`

Still needs follow-up:

- Project owner should relaunch the visible preview and confirm the Create and
  Publish panes are now understandable and readable.
