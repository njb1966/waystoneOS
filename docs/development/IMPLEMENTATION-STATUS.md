# WaystoneOS Implementation Status

Status: Current as of 2026-07-21

This file records what exists in the repository now. It should be updated whenever a planning contract becomes implementation.

Current alignment marker:

- [PHASE-0-0.1-ALIGNMENT.md](PHASE-0-0.1-ALIGNMENT.md)

## Current Implementation

The repository now contains a minimal Rust workspace:

```text
Cargo.toml
crates/project-format/
crates/project-service/
crates/host-service/
crates/identity-service/
crates/publish-plan/
crates/publish-service/
crates/publication-history/
crates/host-identity/
crates/audio-metadata/
crates/audio-service/
crates/cli-output/
cli/project/
cli/publish/
cli/host/
cli/identity/
cli/record/
cli/listen/
cli/way/
ui/workspace-qt/
services/projectd/
services/publishd/
services/hostd/
services/identityd/
services/audiod/
session/
```

## Project Format Crate

Implemented in:

```text
crates/project-format/
```

Current behavior:

- Loads schema-1 `.wayproject/project.toml`
- Inspects project identity and content paths
- Validates supported schema
- Validates supported project type
- Rejects absolute portable paths
- Rejects `..` path traversal
- Checks required content root
- Checks required content index
- Checks supported publish target methods
- Checks duplicate publish target names
- Creates a minimal `.wayproject`
- Creates audio/feed scaffold defaults for `audio-series` and `mixed-publication` projects
- Lists projects with bounded category-depth discovery

Current tests cover:

- Valid minimal capsule fixture
- Missing content index
- Path traversal rejection
- Absolute path rejection
- Project inspection
- Minimal project creation
- Audio-capable project creation defaults
- Removable publish-target metadata setup
- Invalid project ID rejection
- Duplicate publish-target rejection
- Bounded project listing

## Project CLI

Implemented in:

```text
cli/project/
```

Current commands:

```text
project create [--json] PARENT ID NAME TYPE
project target add-removable [--json] PATH NAME EXPORT_PATH
project list [--json] ROOT
project inspect [--json] PATH
project validate [--json] PATH
```

Current behavior:

- Human-readable output
- JSON output
- Exit code `0` for success
- Exit code `2` for usage errors
- Exit code `3` for validation failure
- Basic error reporting with secret-safe messages

## Project Service Crate

Implemented in:

```text
crates/project-service/
```

Current behavior:

- Wraps project-format operations behind request/response structs
- Provides a service boundary for create/list/inspect/validate
- Provides a service boundary for adding removable publish-target metadata
- Does not implement D-Bus activation

Current tests cover:

- Create, validate, and list through the service wrapper

## Host Service Crate

Implemented in:

```text
crates/host-service/
```

Current behavior:

- Wraps host metadata operations behind request/response structs
- Provides a service boundary for list/inspect/validate
- Exposes list/inspect/validate through `waystone-hostd` D-Bus adapter
- Provides repo-local D-Bus service and systemd user unit activation artifacts

Current tests cover:

- List and validate through the service wrapper

## Identity Service Crate

Implemented in:

```text
crates/identity-service/
```

Current behavior:

- Wraps identity metadata operations behind request/response structs
- Provides a service boundary for list/inspect/validate
- Exposes list/inspect/validate through `waystone-identityd` D-Bus adapter
- Provides repo-local D-Bus service and systemd user unit activation artifacts

Current tests cover:

- List and validate through the service wrapper

## Publishing Plan Crate

Implemented in:

```text
crates/publish-plan/
```

Current behavior:

- Loads and validates a `.wayproject`
- Selects a named publish target
- Builds a non-mutating dry-run plan
- Lists publishable content, feed files, and published audio files
- Reports configured feed readiness, feed XML existence, prepared feed-entry count, invalid feed-entry count, and per-invalid-sidecar diagnostics
- Resolves local host metadata when `--hosts` is provided
- Resolves local identity metadata when `--identities` is provided
- Marks dry-run plans blocked when required host or identity metadata is missing or invalid
- Compares local publishable files against a caller-supplied remote-state manifest when provided
- Classifies dry-run changes as upload, delete, and skip; update remains reserved until remote metadata includes hashes
- Reports comparison metadata in the dry-run plan
- Builds a non-mutating transfer intent from immediate validation plus dry-run
  state
- Reports execution readiness, blocking issues, required confirmations, change
  buckets, comparison metadata, host/identity resolution summaries, and the
  future completed-history directory without executing transfer
- Builds a non-mutating removable executor preparation plan with a bounded
  local destination root and per-file source/destination operation records
- Blocks the removable executor preparation plan for unsupported methods,
  existing transfer-intent blockers, and delete operations
- Executes confirmed local/removable file-copy transfers from the removable
  preparation plan
- Copies through a destination-directory temporary file and renames into place
  after the copy completes
- Refuses removable upload overwrites when the destination file already exists
- Refuses stale temporary-copy path collisions before copying starts
- Exports the configured removable destination root's current file path set as
  a local remote-state manifest for later dry-run comparison
- Writes completed-history records from removable executor results, including
  completed, failed, and partial copy-time outcomes
- Does not execute remote deletions
- Does not execute planned delete operations for removable targets yet
- Does not perform SSH-family transfers
- Does not access credentials
- Does not probe SSH host keys

Current tests cover:

- Audio capsule removable-export dry-run
- Feed readiness state for a dry-run with prepared and invalid feed-entry metadata
- Missing publish target rejection
- SSH target host and identity resolution
- Blocked dry-run when host metadata is not provided
- Caller-supplied remote-state comparison and invalid remote-state path rejection
- Ready and blocked transfer-intent reports
- Ready removable executor preparation plans, unsupported-method blockers, and
  delete-operation blockers
- Removable destination-state manifest export and reuse in dry-run comparison
- Confirmed removable file-copy execution, completed-history writing, partial
  copy-time failure history, missing confirmation rejection, upload-overwrite
  rejection, and temporary-copy collision rejection

## Publish CLI

Implemented in:

```text
cli/publish/
```

Current command:

```text
publish --export-remote-state --project PATH --target NAME [--output PATH] [--json]
publish --export-removable-state --project PATH --target NAME [--output PATH] [--json]
publish --inspect-remote-state --remote-state PATH [--json]
publish --execute-removable --project PATH --target NAME --date DATE --confirm-transfer [--remote-state PATH] [--json]
publish --prepare-removable-execution --project PATH --target NAME [--remote-state PATH] [--json]
publish --transfer-intent --project PATH --target NAME [--hosts ROOT] [--identities ROOT] [--remote-state PATH] [--json]
publish --validate --project PATH --target NAME [--hosts ROOT] [--identities ROOT] [--remote-state PATH] [--json]
publish --dry-run --project PATH --target NAME [--hosts ROOT] [--identities ROOT] [--remote-state PATH] [--json]
publish --planned-history --project PATH --target NAME --date DATE [--hosts ROOT] [--identities ROOT] [--remote-state PATH] [--json]
publish --save-planned-history-preview --project PATH --target NAME --date DATE [--hosts ROOT] [--identities ROOT] [--remote-state PATH] [--json]
publish --list-planned-history-previews --project PATH [--json]
publish --read-planned-history-preview --project PATH --preview PATH [--json]
publish --completed-history --project PATH --target NAME --date DATE --transfer-result completed|failed|partial|skipped --verification-result not-run|passed|failed --rollback-available true|false --rollback-notes TEXT [--hosts ROOT] [--identities ROOT] [--remote-state PATH] [--json]
publish --save-completed-history --project PATH --target NAME --date DATE --transfer-result completed|failed|partial|skipped --verification-result not-run|passed|failed --rollback-available true|false --rollback-notes TEXT [--hosts ROOT] [--identities ROOT] [--remote-state PATH] [--json]
publish --list-completed-history --project PATH [--json]
publish --read-completed-history --project PATH --record PATH [--json]
```

Current behavior:

- Human-readable dry-run transfer plan
- Human-readable publication readiness validation report
- Human-readable non-mutating transfer intent report
- Human-readable non-mutating removable executor preparation plan
- Human-readable confirmed removable execution result
- JSON publication readiness validation report with `valid`, `blocked`, `errors`, and `warnings`
- JSON transfer intent report with `execution_ready`, `blocked_reasons`,
  `confirmations`, `host_resolution`, `identity_resolution`, `comparison`,
  `changes`, and `history.completed_directory`
- JSON removable executor preparation report with `execution_ready`,
  `blocked_reasons`, `destination_root`, per-file `operations`, and
  `history.completed_directory`
- Removable executor preparation reports local source/destination paths but
  does not copy files, delete files, or write completed history
- JSON removable execution result with `transfer_result`,
  `verification_result`, per-file copy results, optional per-file error text,
  bytes copied, completed-history path, and completed-history TOML
- `publish --execute-removable` requires `--confirm-transfer`, copies upload
  and update files into the configured removable destination root, refuses
  upload overwrites, uses temporary copy files before renaming into place,
  writes completed history from executor results, records failed/partial
  copy-time outcomes, exits nonzero when transfer result is not `completed`,
  and leaves verification as `not-run`
- Publish validation checks project validation, host and identity resolution, enabled-feed readiness, invalid feed-entry sidecars, empty file-change plans, and required confirmations
- Feed readiness in dry-run JSON and human output
- Optional local remote-state comparison through `--remote-state PATH`
- Dry-run JSON includes `comparison` metadata and populated `upload`, `update`, `delete`, and `skip` arrays
- Local remote-state manifest export from a selected project's publishable path set
- Local removable destination-state manifest export from a selected removable
  target's configured destination root
- Local remote-state manifest inspection using the same path parser as dry-run comparison
- Remote-state export refuses to overwrite an existing output file
- Human-readable planned publication history record
- Local planned history preview save under `history/previews/` inside the selected project
- Read-only saved planned history preview listing from `history/previews/` inside the selected project
- Read-only saved planned history preview detail loading constrained to the selected project's `history/previews/` directory
- Human-readable completed publication history result record generation from explicit result fields
- Local completed history save under `history/completed/` inside the selected project
- Read-only saved completed history listing from `history/completed/` inside the selected project
- Read-only saved completed history detail loading constrained to the selected project's `history/completed/` directory
- Completed-history result validation for transfer result and verification result fields
- JSON output
- Exit code `0` for success
- Exit code `2` for usage errors
- No remote probing or mutation

## CLI Output Helper Crate

Implemented in:

```text
crates/cli-output/
```

Current behavior:

- Provides shared JSON string escaping
- Provides shared optional-string and string-array JSON fragments
- Provides the standard JSON error envelope
- Provides shared command error printing for human and JSON CLI modes
- Does not introduce a full JSON serialization dependency

## Publication History Crate

Implemented in:

```text
crates/publication-history/
```

Current behavior:

- Builds planned publication history records from dry-run plans
- Builds completed publication history result records from dry-run plans plus explicit result fields
- Renders inspectable TOML
- Marks transfer result as `planned`
- Marks verification result as `not-run`
- Writes planned previews under project `history/previews/`
- Writes completed history records under project `history/completed/`
- Lists and reads planned previews constrained to project `history/previews/`
- Lists and reads completed records constrained to project `history/completed/`

Current tests cover:

- Planned history generation from SSH dry-run
- Completed history generation from SSH dry-run plus explicit result fields
- Removable executor preparation JSON contract and unsupported-method blocker
- Removable executor execution JSON contract, copied file results, and
  missing-confirmation rejection
- TOML rendering shape
- Planned preview write, list, read, and outside-directory rejection
- Completed history write, list, read, and outside-directory rejection

## Publish Service Crate

Implemented in:

```text
crates/publish-service/
```

Current behavior:

- Wraps publish dry-run preview behind request/response structs
- Builds publication readiness validation reports
- Builds planned publication history records
- Builds completed publication history result records from explicit result fields
- Saves, lists, and reads completed history records under project `history/completed/`
- Passes optional caller-supplied remote-state manifests into dry-run preview and validation planning
- Builds non-mutating transfer-intent reports from validation and dry-run state
- Builds non-mutating removable executor preparation plans from transfer intent
  and dry-run state
- Executes confirmed removable file-copy transfers from the preparation plan
  through destination-directory temporary files and writes completed-history
  records from executor results, including completed, failed, and partial
  copy-time outcomes
- Preserves blocked dry-run state
- Exposes preview, publication readiness validation, read-only transfer-intent
  reporting, confirmed removable execution, planned-history generation, and
  completed-history result-record generation/save/list/read through
  `waystone-publishd` D-Bus adapter
- Provides repo-local D-Bus service and systemd user unit activation artifacts
- Does not perform remote mutation or delete execution

Current tests cover:

- SSH preview, publication readiness validation, transfer-intent reporting,
  removable executor preparation planning, removable file-copy execution,
  partial copy-time failure history, temporary-copy collision handling,
  planned-history generation, completed-history result generation, and
  completed-history save/list/read
  through the service wrapper
- `waystone-publishd` private-session-bus smoke coverage for preview,
  validation, transfer-intent reporting, confirmed removable execution,
  planned-history, completed-history generation/save/list/read, duplicate owner
  rejection, and invalid request handling

## Host and Identity Crate

Implemented in:

```text
crates/host-identity/
```

Current behavior:

- Loads host TOML records
- Loads identity TOML records
- Lists host records from a directory
- Lists identity records from a directory
- Validates host IDs
- Validates host service trust states
- Validates identity IDs
- Validates SSH public-key shape
- Validates `workspace-secret:` private-key references
- Detects private-key material markers in identity records
- Does not store secrets
- Does not probe remote hosts
- Does not unlock credentials

Current tests cover:

- Valid host example
- Valid identity example
- Invalid trust state
- Private-key material marker detection
- Host listing
- Identity listing

## Audio Metadata Crate

Implemented in:

```text
crates/audio-metadata/
```

Current behavior:

- Creates recording metadata sidecars for existing project-local master and publication-copy files
- Updates existing recording metadata sidecars in place while preserving recording ID, sidecar path, and optional measurement fields
- Captures mono 48 kHz PCM WAV masters under configured project `[audio].masters` roots from explicit `ffmpeg` input sources
- Creates Opus publication-copy files from existing project-local master files through `ffmpeg/libopus`
- Creates feed-entry metadata sidecars under `feeds/entries/` from existing recording metadata
- Updates existing feed-entry metadata sidecars from current recording metadata
- Validates publication-copy and feed-entry handoff metadata in project context
- Loads audio metadata TOML sidecars
- Lists recording metadata from a directory
- Validates recording IDs
- Validates required title
- Validates project-relative master, published, and feed paths
- Refuses metadata sidecar paths that escape the project root
- Refuses to overwrite existing metadata, feed-entry sidecars, captured masters, and publication-copy outputs
- Generates minimal Atom feed XML from validated `feeds/entries/*.toml` sidecars and preserves unrelated existing Atom entries
- Validates positive channel count and sample rate when present
- Warns on unusual MIME type shape
- Does not copy audio files
- Does not generate non-Atom feeds or merge remote feed state
- Does not inspect real audio codecs
- Does not enumerate audio devices

Current tests cover:

- Metadata sidecar creation
- Metadata sidecar update/replacement
- WAV master capture
- Encoded Opus publication-copy export
- Feed-entry sidecar preparation
- Feed-entry sidecar update/replacement
- Minimal Atom feed XML generation and local Atom entry merge/update
- Publication-copy and feed-entry handoff validation
- Valid audio metadata example
- Recording metadata listing
- Invalid upward path rejection

## Audio Service Crate

Implemented in:

```text
crates/audio-service/
```

Current behavior:

- Wraps audio metadata operations behind request/response structs
- Provides a service boundary for attach/update/capture/export-opus/prepare-feed-entry/update-feed-entry/validate-publication/validate-feed-entry/generate-feed/list/inspect/validate
- Exposes attach/update/capture/export-opus/prepare-feed-entry/update-feed-entry/validate-publication/validate-feed-entry/generate-feed/list/inspect/validate through `waystone-audiod` D-Bus adapter
- Provides repo-local D-Bus service and systemd user unit activation artifacts
- Does not enumerate audio devices, play audio, or merge remote feed state

Current tests cover:

- List and validate through the service wrapper
- Recording metadata attachment through the service wrapper
- Recording metadata update through the service wrapper
- WAV master capture through the service wrapper
- Encoded Opus publication-copy export through the service wrapper
- Feed-entry metadata preparation through the service wrapper
- Feed-entry metadata update through the service wrapper
- Minimal Atom feed XML generation and local Atom entry merge/update through the service wrapper
- Publication-copy and feed-entry handoff validation through the service wrapper
- Local audio/feed operations through the `waystone-audiod` D-Bus smoke path

## Host CLI

Implemented in:

```text
cli/host/
```

Current commands:

```text
host list [--json] ROOT
host inspect [--json] PATH
host validate [--json] PATH
```

## Identity CLI

Implemented in:

```text
cli/identity/
```

Current commands:

```text
identity list [--json] ROOT
identity inspect [--json] PATH
identity validate [--json] PATH
```

## Record CLI

Implemented in:

```text
cli/record/
```

Current commands:

```text
record attach [--json] PROJECT ID TITLE MASTER PUBLISHED FEED ENTRY_ID MIME_TYPE
record update [--json] PROJECT RECORDING_ID TITLE MASTER PUBLISHED FEED ENTRY_ID MIME_TYPE
record capture [--json] PROJECT MASTER DURATION_SECONDS INPUT_FORMAT INPUT
record export-opus [--json] PROJECT MASTER PUBLISHED PRESET
record prepare-feed-entry [--json] PROJECT RECORDING_ID UPDATED SUMMARY
record update-feed-entry [--json] PROJECT RECORDING_ID UPDATED SUMMARY
record validate-publication [--json] PROJECT RECORDING_ID
record validate-feed-entry [--json] PROJECT RECORDING_ID
record generate-feed [--json] PROJECT
record list [--json] ROOT
record inspect [--json] PATH
record validate [--json] PATH
```

`record attach` creates a metadata sidecar under the selected project's
configured `[audio].metadata` root for existing project-relative master and
publication-copy files. It records feed enclosure handoff metadata but does not
copy files, generate feeds, transcode audio, or overwrite an existing sidecar.

`record update` rewrites an existing metadata sidecar under the selected
project's configured `[audio].metadata` root. `RECORDING_ID` selects the
existing sidecar; the sidecar path and embedded `recording.id` are preserved.
The command replaces title, master, publication-copy, feed, entry ID, and MIME
fields, preserves optional technical measurements when present, requires the
new master and publication-copy files to exist, and does not edit audio or
prepared feed-entry sidecars.

`record capture` writes a WAV master under the selected project's configured
`[audio].masters` root from an explicit `ffmpeg` input format and input source.
It records for a bounded duration, writes mono 48 kHz PCM WAV, refuses to
overwrite an existing master, and reports the capture engine plus technical
measurements in JSON output. It does not enumerate devices, attach metadata,
export publication copies, generate feeds, or publish remotely.

`record export-opus` validates an existing project-local master file and writes
an encoded `.opus` publication-copy output for the requested project-relative
path through `ffmpeg/libopus`. It refuses to overwrite an existing publication
copy and reports `engine = "ffmpeg"` in JSON output. It does not capture audio,
edit metadata sidecars, or publish remotely.

`record prepare-feed-entry` creates a feed-entry metadata sidecar under
`feeds/entries/` for an existing recording sidecar. It requires the recording
sidecar to include published audio and publication fields and requires the
published audio file to exist. It does not generate or update feed XML by
itself.

`record update-feed-entry` rewrites an existing feed-entry metadata sidecar
under `feeds/entries/` for an existing recording sidecar. It refreshes entry
and enclosure fields from the current recording metadata and replaces only the
feed-entry `updated` and `summary` inputs from the command line. It does not
create missing feed-entry sidecars or generate feed XML by itself.

`record validate-publication` validates an existing recording sidecar in project
context, including referenced master and publication-copy files. `record
validate-feed-entry` validates a prepared feed-entry sidecar against its
recording metadata, enclosure file, and sibling feed-entry IDs.

`record generate-feed` reads the selected project's enabled Atom `[feed]`
configuration, validates prepared feed-entry sidecars, and atomically writes the
configured feed XML path. If the configured feed already contains Atom entries,
the command replaces entries with matching prepared sidecar IDs and preserves
unrelated existing entries in their current XML form. It does not support RSS,
copy audio, transcode audio, merge remote feeds, or publish remotely.

## Listen CLI

Implemented in:

```text
cli/listen/
```

Current command:

```text
listen library [--json] ROOT
```

## Way CLI

Implemented in:

```text
cli/way/
```

Current command:

```text
way [help|--help]
```

Current behavior:

- Lists current core commands
- Lists current service binaries
- Returns exit code `2` for unsupported arguments
- Does not dispatch to subcommands yet

## Workspace Qt Scaffold

Implemented in:

```text
ui/workspace-qt/
```

Current behavior:

- Builds as a standalone Qt 6 C++ CMake project when Qt 6 development files are installed
- Renders the first Waystone Workspace frame
- Provides a top menu bar, workspace selectors, left activity navigation, stacked main panes, and status bar
- Uses the existing `project` CLI JSON output for Create-pane create, removable target setup, list, inspect, and validate
- Creates projects under the configured projects root, adds a default removable `export` target, refreshes the project list, and opens the new project in the editor
- Creates `audio-series` and `mixed-publication` projects with audio/feed scaffold defaults through the same `project create --json` path
- Lets the Create pane add removable publish targets to the selected project
- Loads, edits, saves, validates, previews, and locally link-checks the selected project's content index file in the Create pane
- Lists files under the selected project's content root in the Create pane without changing the editable content index binding
- Filters the Create pane content-root file list by relative path or full path
- Shows read-only detail for the selected Create pane content-root file, including whether it is the editable content index
- Uses the existing `record` and `listen` CLI JSON output for Create-pane recording list, inspect, validate, playable state, and local metadata sidecar attachment
- Uses `record capture --json` in the Create pane to create WAV masters from explicit `ffmpeg` input sources
- Uses `record export-opus --json` in the Create pane to create encoded Opus publication copies from existing project-local master files
- Uses `record attach --json` in the Create pane to create audio metadata sidecars for existing project-local master and publication-copy files when the selected project has audio metadata configured
- Uses `record update --json` in the Create pane to update existing recording metadata sidecars for selected projects
- Uses `record prepare-feed-entry --json`, `record update-feed-entry --json`, `record validate-publication --json`, and `record validate-feed-entry --json` in the Create pane to prepare, update, and validate local feed-entry handoff metadata for attached recordings
- Uses `record generate-feed --json` in the Create pane to generate local Atom feed XML from prepared feed-entry sidecars
- Uses configured local projects plus existing `publish` CLI JSON output for read-only Publish-pane dry-run previews
- Uses `publish --validate --json` in the Publish pane to show a read-only publication readiness report
- Uses `publish --transfer-intent --json` in the Publish pane to show a
  read-only execution-readiness report
- Uses `publish --prepare-removable-execution --json` in the Publish pane to
  show a read-only removable execution readiness report with destination-root
  and per-file operation paths
- Derives Publish-pane target choices from `project inspect --json` instead of hard-coded project IDs
- Shows Publish-pane preview status as ready, blocked, failed, no project, no target, and feed readiness when feed metadata is configured
- Shows invalid feed-entry diagnostic paths and validation issue text in Publish-pane dry-run detail
- Shows read-only validation detail for a selected invalid feed-entry diagnostic through `record validate-feed-entry --json`
- Lets a selected invalid feed-entry diagnostic open the matching project and derived recording ID in the Create pane
- Filters visible Publish-pane projects by project name, ID, type, path, or target names
- Shows a compact per-target Publish-pane overview for the selected project using read-only dry-run status, method, change count, verification count, and destination
- Accepts an optional local remote-state manifest path in the Publish pane and passes it through preview, validation, transfer-intent, removable execution readiness, planned-history, saved-preview, and target-overview CLI calls
- Exports a selected removable target's destination-state manifest from the
  Publish pane through `publish --export-removable-state --json`, saves it
  under the selected project's `history/previews/` directory, and loads that
  project-local path into the existing remote-state comparison field
- Renders Publish-pane comparison metadata plus upload, update, delete, and skip dry-run buckets
- Lets Publish-pane target overview row selection update the active target and refresh the existing preview/history panes
- Shows planned publication history file-action summary and raw TOML generated by `publish --planned-history --json`
- Saves planned history previews under the selected project `history/previews/` directory through `publish --save-planned-history-preview --json`
- Lists saved planned history previews for the selected project through `publish --list-planned-history-previews --json`
- Loads selected saved planned history preview TOML through `publish --read-planned-history-preview --json`
- Preserves the selected saved preview row across refreshes when the preview still exists
- Compares generated planned-history TOML against the selected saved preview and reports the first differing line
- Filters saved preview records by filename or path inside the already loaded project-local preview list
- Lists saved completed history records for the selected project through `publish --list-completed-history --json`
- Loads selected saved completed history record TOML through `publish --read-completed-history --json`
- Filters saved completed history records by filename or path inside the already loaded project-local completed list
- Uses the existing `host` and `identity` CLI JSON output for read-only Operate-pane list, inspect, and validate
- Keeps command execution and JSON parsing in `ui/workspace-qt/src/cli_adapter.*`
- Keeps page construction in `ui/workspace-qt/src/workspace_pages.*`
- Keeps local Workspace root configuration in `ui/workspace-qt/src/workspace_config.*`
- Provides example root configuration in `ui/workspace-qt/workspace.example.ini`
- Shows active configured roots in the Explore pane
- Reads explicit `--config`, user app config, or repository defaults in that order
- Lets the Explore pane write persistent user root settings
- Preflights missing configured roots before running pane CLIs
- Provides `--check-roots` diagnostics for bad config paths and missing configured roots
- Provides diagnostic project create/save, recording attachment/feed-entry/feed-generation, and Publish-pane target/status/export-state smoke modes for temporary workspace roots
- Uses static placeholder resource data for Explore
- Mutates only persistent user root settings, minimal project directories under the configured projects root, removable publish target metadata, selected local project content index files, planned preview records, removable-state manifest helper files under project `history/previews/`, selected project audio metadata sidecars, selected project feed-entry sidecars, and selected project feed XML
- Does not expose `publish --execute-removable` in the Qt UI yet
- Does not call Rust crates directly, D-Bus, sibling apps, audio devices, or remote services

## Session Layout

Implemented in:

```text
session/
scripts/session-layout-smoke.sh
```

Current behavior:

- Provides a repo-local XDG Wayland session entry source file at `session/waystone.desktop`
- Provides a repo-local launcher wrapper source file at `session/waystone-session`
- Provides a repo-local install-layout manifest at `session/install-layout.toml`
- Defines future Debian preview target paths for the session entry, wrapper,
  Workspace binary, service binaries, D-Bus service files, and systemd user
  units
- Lets the wrapper execute a Workspace binary selected by `WAYSTONE_WORKSPACE_BIN`
- Falls back to `/usr/bin/waystone-workspace` when no override is provided
- Passes optional `WAYSTONE_REPO_ROOT` and `WAYSTONE_WORKSPACE_CONFIG` values
  to the Workspace as `--repo-root` and `--config`
- Fails with exit code `127` and a clear diagnostic when the selected Workspace
  binary is not executable
- Does not install files outside the repository, register with a display
  manager, start services, call D-Bus, create a bootable image, or launch
  sibling applications

## Project Service

Implemented in:

```text
services/projectd/
```

Current state:

- Runs as a direct D-Bus session service when launched manually
- Owns `org.waystone.Project1` at `/org/waystone/Project`
- Implements `CreateProject`, `ListProjects`, `InspectProject`, and `ValidateProject`
- Requests single-owner bus name behavior; duplicate daemon instances fail quickly
- Provides repo-local D-Bus service and systemd user unit activation artifacts
- Uses `zbus` 5.13.1 pinned for Rust 1.85.0 compatibility
- D-Bus autostart is smoke-tested through a generated temporary service file
- Systemd user unit syntax is smoke-tested through a generated temporary daemon path
- Service contract documented in `docs/architecture/PROJECT-SERVICE.md`
- Uses `crates/project-service/` as its internal boundary

## Publish Service

Implemented in:

```text
services/publishd/
```

Current state:

- Runs as a direct D-Bus session service when launched manually
- Owns `org.waystone.Publish1` at `/org/waystone/Publish`
- Implements `PreviewPublication`, `ValidatePublication`, `TransferIntent`, `BuildPlannedHistory`, `BuildCompletedHistory`, `SaveCompletedHistory`, `ListCompletedHistory`, and `ReadCompletedHistory`
- Requests single-owner bus name behavior; duplicate daemon instances fail quickly
- Provides repo-local D-Bus service and systemd user unit activation artifacts
- D-Bus autostart is smoke-tested through a generated temporary service file
- Systemd user unit syntax is smoke-tested through a generated temporary daemon path
- Accepts caller-supplied local remote-state manifests for preview and
  transfer-intent comparison
- Does not expose removable execution over D-Bus yet
- Does not probe remote hosts, perform SSH-family transfers, execute
  deletions, unlock credentials, or verify remote results
- Uses `crates/publish-service/` as its internal boundary

## Host and Identity Services

Implemented in:

```text
services/hostd/
services/identityd/
```

Current state:

- Run as direct D-Bus session services when launched manually
- `waystone-hostd` owns `org.waystone.Host1` at `/org/waystone/Host`
- `waystone-identityd` owns `org.waystone.Identity1` at `/org/waystone/Identity`
- Implement list, inspect, validate, and structured invalid-request responses
- Request single-owner bus name behavior; duplicate daemon instances fail quickly
- Provide repo-local D-Bus service and systemd user unit activation artifacts
- D-Bus autostart is smoke-tested through generated temporary service files
- Systemd user unit syntax is smoke-tested through generated temporary daemon paths
- Metadata logic remains in `crates/host-identity/`
- `hostd` uses `crates/host-service/` as its internal boundary
- `identityd` uses `crates/identity-service/` as its internal boundary

## Audio Service

Implemented in:

```text
services/audiod/
```

Current state:

- Runs as a direct D-Bus session service when launched manually
- Owns `org.waystone.Audio1` at `/org/waystone/Audio`
- Implements `ListRecordings`, `InspectRecording`, `ValidateRecording`,
  `AttachRecording`, `UpdateRecording`, `CaptureRecording`, `ExportOpus`,
  `PrepareFeedEntry`, `UpdateFeedEntry`, `ValidatePublication`,
  `ValidateFeedEntry`, and `GenerateFeed`
- Requests single-owner bus name behavior; duplicate daemon instances fail quickly
- Provides repo-local D-Bus service and systemd user unit activation artifacts
- D-Bus autostart is smoke-tested through a generated temporary service file
- Systemd user unit syntax is smoke-tested through a generated temporary daemon path
- Audio device enumeration is not implemented
- Uses `crates/audio-service/` as its internal boundary

## CLI Integration Tests

Implemented in each CLI crate under `cli/*/tests/`.

Current tests cover:

- `project validate` reports invalid fixtures
- `publish --dry-run --json` reports resolved host and identity metadata
- `publish --dry-run --json` reports configured feed readiness, prepared feed-entry counts, and invalid feed-entry diagnostics
- `publish --validate --json` reports valid/blocked publication readiness, issue codes, and warnings
- `publish --planned-history --json`, planned preview save/list/read, `publish --completed-history --json`, and completed history save/list/read report stable JSON contracts and reject reads outside the selected project history directories
- `host validate` rejects invalid trust state
- `identity validate` rejects private-key material
- `record validate` rejects invalid audio paths
- `record capture --json` writes a WAV master through an explicit `ffmpeg` input source, `record export-opus --json` writes an encoded Opus publication copy through `ffmpeg/libopus`, `record attach --json` creates recording metadata, `record update --json` rewrites existing recording metadata, `record prepare-feed-entry --json` creates feed-entry metadata, `record update-feed-entry --json` rewrites existing feed-entry metadata, `record validate-publication --json` plus `record validate-feed-entry --json` validate local audio publication handoff, and `record generate-feed --json` writes a local Atom feed
- `listen library --json` lists recording metadata
- `way --help` lists current core commands

## Fixtures and Examples

Examples:

```text
examples/projects/minimal-capsule.wayproject/
examples/projects/audio-capsule.wayproject/
examples/projects/ssh-capsule.wayproject/
examples/connections/hosts/offgridholdout.toml
examples/connections/identities/nick-pub.toml
```

Invalid fixtures:

```text
tests/fixtures/projects/invalid-missing-index.wayproject/
tests/fixtures/projects/invalid-path-traversal.wayproject/
tests/fixtures/projects/invalid-absolute-path.wayproject/
tests/fixtures/hosts/invalid-trust/host.toml
tests/fixtures/identities/private-key-leak/identity.toml
tests/fixtures/audio/invalid-path/field-note.toml
```

## Verification Commands

Current verification:

```text
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Qt scaffold verification status:

```text
cmake -S ui/workspace-qt -B /tmp/waystone-workspace-qt-build
cmake --build /tmp/waystone-workspace-qt-build
QT_QPA_PLATFORM=offscreen timeout 5s /tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS
scripts/workspace-qt-smoke.sh
scripts/cli-json-contract-smoke.sh
scripts/workspace-qt-project-smoke.sh
scripts/projectd-dbus-smoke.sh
scripts/publishd-dbus-smoke.sh
scripts/host-identity-dbus-smoke.sh
scripts/audiod-dbus-smoke.sh
scripts/projectd-dbus-activation-smoke.sh
scripts/publishd-dbus-activation-smoke.sh
scripts/host-identity-dbus-activation-smoke.sh
scripts/audiod-dbus-activation-smoke.sh
scripts/projectd-systemd-unit-smoke.sh
scripts/publishd-systemd-unit-smoke.sh
scripts/host-identity-systemd-unit-smoke.sh
scripts/audiod-systemd-unit-smoke.sh
scripts/session-layout-smoke.sh
```

Local results through 2026-07-21: Qt 6 was discoverable after installing
`qt6-base-dev`; configure and build passed. The offscreen Qt startup smoke
script launched the app successfully and verified root handling. The focused Qt
project smoke created a minimal project in a generated `/tmp` workspace root,
added a removable `export` target, loaded its content index, saved edited
Gemtext through the Qt CLI adapter, verified the Create pane content-file list
reported `index.gmi`, verified the Create pane content-file filter isolated
`index.gmi`, verified selected-file detail for index and non-index content
files, verified `project inspect --json` reported the target, validated the
result, and verified a removable publish dry-run preview. The same smoke
creates an audio-capable temporary project, verifies that project creation
supplies audio/feed scaffold defaults, verifies that the Create-pane recording
capture controls create a WAV master from an explicit `ffmpeg` input source,
verifies that the Create-pane recording attachment controls create an
inspectable metadata sidecar for project-local audio files, verifies that the
Create-pane recording update controls rewrite that sidecar for revised
project-local files, prepares a feed-entry sidecar, verifies publication/feed-
entry validation status, generates feed XML, verifies that the generated Atom
feed contains the expected entry, and verifies that the Publish pane reports
the generated feed as ready with one prepared entry. The same focused smoke
also creates a separate temporary project with multiple publish targets and
verifies that the Publish pane target selector drives ready, blocked, project
filtering, per-target overview rows, overview-row target selection,
planned-history summary, raw planned-history record preview, local remote-state
comparison detail, removable destination-state export into a project-local
manifest, saved planned-history preview transitions, saved-preview listing,
selected saved-preview detail loading, saved-preview row selection
preservation, generated-vs-saved comparison reporting, saved-preview filtering,
completed-history listing, selected completed-record detail loading, and
completed-record filtering. The Publish pane derives all discovered project
targets into a selector, filters visible projects, reports dry-run preview
state as ready, blocked, failed, no project, no target, feed missing, feed
entries invalid, feed present, or feed ready, shows a compact per-target
overview, accepts an optional local remote-state manifest path, can export a
removable target's destination-state manifest into project `history/previews/`
and load that generated path for comparison, renders comparison metadata and
upload/update/delete/skip buckets, lets overview row selection update the
active target, shows planned publication history file-action grouping plus raw
TOML, saves planned previews under project `history/previews/`, lists saved
preview records, loads selected preview TOML, preserves the selected saved
preview across refreshes, reports first-line differences between generated and
selected saved planned history, filters the visible saved-preview list, lists
saved completed history records, loads selected completed-record TOML, and
filters the visible completed-record list without mutating remotes. The
`publish` CLI also builds completed history result records from explicit result
fields and saves, lists, and reads them under project `history/completed/`. The
CLI JSON contract smoke verifies publication readiness validation,
completed-history generation/save/list/read, `record capture --json`, `record
attach --json`, `record update --json`, `record prepare-feed-entry --json`,
`record update-feed-entry --json`, `record validate-publication --json`,
`record validate-feed-entry --json`, `record generate-feed --json`, and publish
dry-run feed readiness against temporary project data. The projectd D-Bus smoke
script verified create, list, inspect, validate, invalid-request handling,
unavailable-bus failure, and duplicate-owner failure on a private test session
bus. The publishd D-Bus smoke script verified preview, publication readiness
validation, transfer-intent reporting, confirmed removable execution,
planned-history generation, completed-history result-record
generation/save/list/read, invalid-request handling, unavailable-bus failure,
and duplicate-owner failure on a private test session bus. The host/identity
D-Bus smoke script verified list, inspect, validate, invalid-request handling,
unavailable-bus failure, and duplicate-owner failure for both adapters on a
private test session bus. The audiod D-Bus smoke script verified list, inspect,
validate, invalid-request handling, unavailable-bus failure, and duplicate-owner
failure on a private test session bus. The activation smokes verified projectd,
publishd, host/identity, and audiod D-Bus service-file autostart. The systemd
smokes verified projectd, publishd, host/identity, and audiod unit syntax
through temporary paths.
The session layout smoke verified the repo-local session entry, install
manifest, wrapper argument passing through a fake Workspace binary, and clear
failure when the selected Workspace binary is missing.

Local result on 2026-07-20: real `ffmpeg/libopus` Opus publication-copy export
passed Rust tests, clippy with warnings denied, and the CLI JSON contract
smoke. Qt Create-pane export controls passed Qt build and focused Qt project
smoke.
Publish feed diagnostics passed Rust tests, clippy with warnings denied, CLI
JSON contract smoke, publishd D-Bus smoke, and focused Qt project smoke.
Publish feed-entry validation detail passed Qt build and focused Qt project
smoke. Qt recording metadata update controls passed Qt build and focused Qt
project smoke. Feed-entry update command passed Rust tests, clippy with
warnings denied, CLI JSON contract smoke, and audiod D-Bus smoke. Qt
feed-entry update controls passed Qt build and focused Qt project smoke.
Publish-to-Create feed diagnostic handoff passed Qt build and focused Qt
project smoke.
Local Atom feed merge/update passed Rust tests, clippy with warnings denied,
CLI JSON contract smoke, focused Qt project smoke, and broad Qt smoke.
Narrow local recording capture passed Rust tests, clippy with warnings denied,
CLI JSON contract smoke, audiod D-Bus smoke, focused Qt project smoke, broad Qt
smoke, format checks, and git diff whitespace checks. Qt recording capture
controls passed Qt build, focused Qt project smoke, broad Qt smoke, Rust tests,
clippy with warnings denied, CLI JSON contract smoke, format checks, and git
diff whitespace checks.
Completed publication-history result records passed targeted Rust tests for the
history crate, publish service crate, and publish CLI; full-slice verification
is recorded in `docs/development/CHECKPOINT.md`.

Useful CLI smoke checks:

```text
cargo run -q -p waystone-project-cli -- inspect examples/projects/minimal-capsule.wayproject
cargo run -q -p waystone-project-cli -- validate tests/fixtures/projects/invalid-missing-index.wayproject
cargo run -q -p waystone-project-cli -- validate --json examples/projects/minimal-capsule.wayproject
cargo run -q -p waystone-publish-cli -- --dry-run --project examples/projects/audio-capsule.wayproject --target export
cargo run -q -p waystone-publish-cli -- --dry-run --project examples/projects/ssh-capsule.wayproject --target production --hosts examples/connections/hosts --identities examples/connections/identities
cargo run -q -p waystone-host-cli -- list examples/connections/hosts
cargo run -q -p waystone-identity-cli -- validate tests/fixtures/identities/private-key-leak/identity.toml
cargo run -q -p waystone-record-cli -- inspect examples/projects/audio-capsule.wayproject/audio/metadata/field-note.toml
cargo run -q -p waystone-listen-cli -- library --json examples/projects/audio-capsule.wayproject/audio/metadata
cargo run -q -p waystone-way-cli -- --help
cargo build -p waystone-project-cli -p waystone-publish-cli -p waystone-host-cli -p waystone-identity-cli -p waystone-record-cli -p waystone-listen-cli
cmake -S ui/workspace-qt -B /tmp/waystone-workspace-qt-build
cmake --build /tmp/waystone-workspace-qt-build
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS --config /path/to/workspace.ini
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS --no-user-config
/tmp/waystone-workspace-qt-build/waystone-workspace --repo-root /path/to/waystoneOS --check-roots
scripts/workspace-qt-project-smoke.sh
```

## Not Implemented Yet

- Installed D-Bus activation
- Installed long-running `waystone-projectd`
- Installed or activatable long-running `waystone-publishd`
- Installed or activatable long-running `waystone-hostd`
- Installed or activatable long-running `waystone-identityd`
- Installed or activatable long-running `waystone-audiod`
- `way` subcommand dispatch
- Project migration
- Project archive/export
- Project repair
- Remote publication comparison
- Remote transfer execution
- Remote verification
- Persistent host and identity workspace storage
- Secret storage
- SSH host-key probing
- Audio device enumeration
- Audio recording
- Audio playback
- Audio trimming, normalization, or codec export/transcoding beyond Opus publication copies
- Full audio metadata merge editing beyond the narrow `record update` replacement command
- Remote feed merge updates or non-Atom feed generation
- Deeper Workspace actions beyond local inspect, authoring, preview, and feed generation
- Live reload after editing persistent user settings
- Browser, Helm, or Comm integration
