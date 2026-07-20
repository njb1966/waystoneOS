# Waystone Workspace Qt Development

Status: local authoring prototype
Date: 2026-07-19

The first Waystone Workspace UI source lives in:

```text
ui/workspace-qt/
```

It is a Qt 6 C++ scaffold for the operating frame defined in `docs/architecture/WORKSPACE-UI-PLAN.md` and ADR-0012.

## Current Scope

Implemented:

- Top menu bar
- Four workspace selector buttons
- Left activity navigation
- Main stacked workspace panes
- Bottom status bar
- Create-pane adapter using `project` CLI JSON output for create, removable target setup, list, inspect, and validate
- Create-pane project creation form for projects under the configured projects root
- Audio-capable project creation defaults for `audio-series` and `mixed-publication` through `project create --json`
- Default removable `export` target setup for newly created Qt projects
- Create-pane removable target setup for the selected project
- Create-pane editor for the selected project's content index file
- Create-pane read-only content-root file list for the selected project
- Create-pane content-root file filter by relative path or full path
- Create-pane selected content-file detail, including editable-index status
- Local Gemtext preview, link validation, and save validation status for the selected project's content index file
- Recording adapter using `record` and `listen` CLI JSON output
- Create-pane mock publication-copy export controls using `record export-opus --json` for existing project-local master files
- Create-pane recording attachment controls using `record attach --json` for existing project-local master and publication-copy files
- Create-pane feed-entry preparation and validation controls using `record prepare-feed-entry --json`, `record validate-publication --json`, and `record validate-feed-entry --json`
- Read-only Publish-pane adapter using configured local projects and `publish --dry-run --json`
- Publish-pane target selection derived from `project inspect --json` metadata
- Publish-pane ready, blocked, failed, no-project, and no-target preview status display
- Publish-pane planned history summary and raw record preview using `publish --planned-history --json`
- Publish-pane planned history preview save using `publish --save-planned-history-preview --json`
- Publish-pane saved preview listing using `publish --list-planned-history-previews --json`
- Publish-pane selected saved preview detail loading using `publish --read-planned-history-preview --json`
- Publish-pane saved preview row selection preservation across refreshes
- Publish-pane generated-vs-saved planned-history comparison summary
- Publish-pane saved-preview filename/path filter
- Publish-pane per-target overview table using read-only dry-run status, method, upload count, verification count, and destination
- Publish-pane target overview row selection updates the active target and refreshes the existing preview/history panes
- Publish-pane project filter by name, ID, type, path, or target names
- Publish-pane feed readiness reporting from `publish --dry-run --json`
- Publish-pane invalid feed-entry diagnostic paths and issue text from `publish --dry-run --json`
- Publish-pane validation detail action for selected invalid feed-entry diagnostics using `record validate-feed-entry --json`
- Read-only Operate-pane adapters using `host` and `identity` CLI JSON output
- `main.cpp` application frame split from page construction in `workspace_pages.cpp`
- Local Workspace root configuration in `workspace_config.cpp`
- Explore pane active-root display for the loaded Workspace configuration
- Explore pane editing for persistent user root settings
- Missing root preflight errors shown in affected panes
- Create-pane feed XML generation through the local `record generate-feed --json` adapter
- Static placeholder resource data for Explore only

Not implemented:

- Service calls
- D-Bus
- Remote publishing
- Audio device access
- Audio capture, playback, or real codec transcoding
- Browser, Helm, or Comm embedding
- Custom compositor behavior

## Debian Prerequisites

Expected packages:

```text
cmake
g++
qt6-base-dev
```

Do not install packages as part of the repo build. The UI remains a native Debian prototype component with ordinary system dependencies.

The Create, Publish, and Operate panes also expect the current CLIs to be built or installed:

```bash
cargo build -p waystone-project-cli -p waystone-publish-cli -p waystone-host-cli -p waystone-identity-cli -p waystone-record-cli -p waystone-listen-cli
```

When launched from the repository root, the UI will look for matching binaries under `target/debug/` before falling back to commands on `PATH`.

## Build

```bash
cmake -S ui/workspace-qt -B /tmp/waystone-workspace-qt-build
cmake --build /tmp/waystone-workspace-qt-build
```

The scaffold has been verified locally with Qt 6.8.2 after installing `qt6-base-dev`.

## Run

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace
```

If launched from another working directory, pass the repo root:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS
```

If using explicit data roots, pass an INI file:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS --config /path/to/workspace.ini
```

If no `--config` is passed, the UI checks Qt's user app config location for `workspace.ini`. Use this option to ignore user config and force repository defaults:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS --no-user-config
```

To validate configured roots without opening the window:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS --check-roots
```

The binary also has diagnostic smoke modes for project creation/content save behavior, Create-pane recording attachment/feed-entry behavior, and Publish-pane target/status behavior. Prefer the wrapper script below rather than invoking them by hand.

Example config:

```text
ui/workspace-qt/workspace.example.ini
```

The Explore pane can save the same root settings to the user config file. Saved user settings are loaded on the next launch when neither `--config` nor `--no-user-config` is passed.

Supported keys:

```ini
[roots]
projects = examples/projects
hosts = examples/connections/hosts
identities = examples/connections/identities
audio_metadata = examples/projects/audio-capsule.wayproject/audio/metadata
```

Headless startup smoke test:

```bash
scripts/workspace-qt-smoke.sh
```

The smoke test uses Qt's `offscreen` platform and verifies default-root, explicit-config, user-config, missing-config fallback, and missing-root diagnostics.

Focused project create/save, recording attachment/feed-entry, and Publish-pane target/status smoke test:

```bash
scripts/workspace-qt-project-smoke.sh
```

The project smoke test uses a generated `/tmp` workspace root, creates a
minimal project through the Qt CLI adapter, adds a removable export target,
saves edited content through the same adapter, validates the result, verifies
the Create pane content-file list, filter, and selected-file detail, and
verifies a removable publish dry-run preview without touching repository
examples. It also creates an audio-capable temporary project, verifies that
project creation supplies audio/feed scaffold defaults, verifies that the
Create-pane recording export control writes a mock publication copy, verifies
that recording attachment controls create an inspectable metadata sidecar for
project-local audio files, prepares a feed-entry sidecar, verifies
publication/feed-entry validation status, generates feed XML, adds one broken
feed-entry sidecar, and verifies that the Publish pane reports invalid feed
state with a diagnostic path and validation issue text. It also verifies that
the selected feed diagnostic can be validated from the Publish pane. It also
creates a separate temporary project with multiple publish targets and verifies
that the Publish pane target selector drives ready,
blocked, project filtering, per-target overview rows, overview-row target
selection, planned-history summary, raw planned-history record preview, saved
planned-history preview transitions, saved-preview listing, selected saved
preview detail loading, saved-preview row selection preservation,
generated-vs-saved comparison reporting, and saved-preview filtering without
remote publication.

CLI JSON contract smoke test:

```bash
scripts/cli-json-contract-smoke.sh
```

## Next Integration Boundary

The first adapter path is CLI JSON for project, recording, publish preview, host, and identity data, plus local file access for project creation, selected content editing, and Gemtext link validation:

- `project create --json`
- `project target add-removable --json`
- `project list --json`
- `project inspect --json`
- `project validate --json`
- `record export-opus --json`
- `record attach --json`
- `record prepare-feed-entry --json`
- `record validate-publication --json`
- `record validate-feed-entry --json`
- `publish --dry-run --json`
- `publish --planned-history --json`
- `publish --save-planned-history-preview --json`
- `publish --list-planned-history-previews --json`
- `publish --read-planned-history-preview --json`
- `host list --json`
- `host inspect --json`
- `host validate --json`
- `identity list --json`
- `identity inspect --json`
- `identity validate --json`
- `record list --json`
- `record inspect --json`
- `record validate --json`
- `listen library --json`
- minimal project creation under the configured projects root
- selected project content index read/write
- selected project content-root link existence checks

Before moving Qt to a service backend, define the next adapter scope:

- C ABI/service adapter for tighter Rust crate reuse, or
- D-Bus adapter after daemon contracts exist.
