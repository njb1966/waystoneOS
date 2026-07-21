# Waystone Workspace Qt Prototype

Status: local authoring prototype

This directory contains the first Qt 6 C++ scaffold for Waystone Workspace.

It is intentionally separate from the Rust workspace. The UI renders the operating frame and uses existing CLI JSON output through a small Qt adapter layer.

## Boundaries

- No D-Bus dependency yet
- No remote publication
- No Browser, Helm, or Comm embedding
- No domain behavior in widgets
- No generated assets or network access

Current UI data is limited to:

- `project create --json`
- `project target add-removable --json`
- `project list --json`
- `project inspect --json` for project metadata and selected content index location
- `project validate --json`
- `publish --dry-run --json`
- `publish --export-removable-state --json`
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
- `record capture --json`
- `record export-opus --json`
- `record attach --json`
- `record update --json`
- `record prepare-feed-entry --json`
- `record update-feed-entry --json`
- `record validate-publication --json`
- `record validate-feed-entry --json`
- `record inspect --json`
- `record validate --json`
- `listen library --json`
- local creation of projects under the configured projects root
- local audio/feed scaffold defaults for `audio-series` and `mixed-publication`
- local removable publish-target metadata setup
- local reads and writes of the selected project's content index file
- local content-root file listing, filtering, and selected-file detail for the selected project
- local WAV master capture from explicit `ffmpeg` input sources through `record capture --json`
- local Opus publication-copy export from existing project-local master files through `record export-opus --json`
- local audio metadata sidecar attachment for existing project-local master and publication-copy files
- local audio metadata sidecar updates for existing project recordings
- local feed-entry sidecar preparation, update, and validation status for attached project recordings
- local feed XML generation from prepared entries through `record generate-feed --json`
- local Publish-pane invalid feed-entry diagnostic paths and issue text
- local Publish-pane validation detail for selected invalid feed-entry diagnostics
- local Publish-pane handoff from selected invalid feed-entry diagnostics back to Create
- local Gemtext preview rendering in the Create pane
- local Gemtext link validation for external, missing, invalid, and content-root-local links
- local Publish-pane project filtering, target selection, ready/blocked/feed-aware preview status, per-target overview rows, overview-row target selection, removable destination-state export into project-local manifest files, planned history file-action summary, raw planned history preview, project-local planned preview save, saved-preview listing, selected saved-preview detail loading, saved-preview selection preservation, generated-vs-saved comparison reporting, and saved-preview filtering

Future UI data should continue to come from WaystoneOS service contracts, current CLI JSON output, or a narrow adapter approved before implementation.

## Workspace Roots

By default, the UI reads from repository examples:

- `examples/projects`
- `examples/connections/hosts`
- `examples/connections/identities`
- `examples/projects/audio-capsule.wayproject/audio/metadata`

An INI file can override those roots. Explicit `--config` wins. If no explicit config is passed, the UI checks Qt's user app config location for `workspace.ini`; if that file is absent, repository defaults are used.

The Explore pane can write persistent user root settings to the same user config file. Saved settings are picked up on the next launch unless `--config` or `--no-user-config` is used.

Example config:

```text
ui/workspace-qt/workspace.example.ini
```

## Debian Build

Expected packages:

```text
cmake
g++
qt6-base-dev
```

The Create, Publish, and Operate panes expect the current CLIs to be available. During development, build them first:

```bash
cargo build -p waystone-project-cli -p waystone-publish-cli -p waystone-host-cli -p waystone-identity-cli -p waystone-record-cli -p waystone-listen-cli
```

The UI looks for the matching binaries under `target/debug/` before falling back to commands on `PATH`.

Build commands:

```bash
cmake -S ui/workspace-qt -B /tmp/waystone-workspace-qt-build
cmake --build /tmp/waystone-workspace-qt-build
```

Run command from the repository root:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace
```

Or pass the repository root explicitly:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS
```

Run with explicit workspace roots:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS --config /path/to/workspace.ini
```

Ignore user config and use repository defaults:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS --no-user-config
```

Check configured roots without opening the window:

```bash
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS --check-roots
```

Focused project create/save, recording attachment/feed-entry/feed-generation, and Publish-pane target/status smoke check:

```bash
scripts/workspace-qt-project-smoke.sh
```

Repeatable smoke check:

```bash
scripts/workspace-qt-smoke.sh
```

CLI JSON contract smoke check:

```bash
scripts/cli-json-contract-smoke.sh
```
