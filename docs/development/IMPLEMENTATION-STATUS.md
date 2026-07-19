# WaystoneOS Implementation Status

Status: Current as of 2026-07-19

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
- Resolves local host metadata when `--hosts` is provided
- Resolves local identity metadata when `--identities` is provided
- Marks dry-run plans blocked when required host or identity metadata is missing or invalid
- Reports planned uploads only
- Does not compare remote state
- Does not perform transfers
- Does not delete remote files
- Does not access credentials
- Does not probe SSH host keys

Current tests cover:

- Audio capsule removable-export dry-run
- Missing publish target rejection
- SSH target host and identity resolution
- Blocked dry-run when host metadata is not provided

## Publish CLI

Implemented in:

```text
cli/publish/
```

Current command:

```text
publish --dry-run --project PATH --target NAME [--hosts ROOT] [--identities ROOT] [--json]
publish --planned-history --project PATH --target NAME --date DATE [--hosts ROOT] [--identities ROOT] [--json]
publish --save-planned-history-preview --project PATH --target NAME --date DATE [--hosts ROOT] [--identities ROOT] [--json]
publish --list-planned-history-previews --project PATH [--json]
publish --read-planned-history-preview --project PATH --preview PATH [--json]
```

Current behavior:

- Human-readable dry-run transfer plan
- Human-readable planned publication history record
- Local planned history preview save under `history/previews/` inside the selected project
- Read-only saved planned history preview listing from `history/previews/` inside the selected project
- Read-only saved planned history preview detail loading constrained to the selected project's `history/previews/` directory
- JSON output
- Exit code `0` for success
- Exit code `2` for usage errors
- No remote mutation

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
- Renders inspectable TOML
- Marks transfer result as `planned`
- Marks verification result as `not-run`
- Does not write completed history records

Current tests cover:

- Planned history generation from SSH dry-run
- TOML rendering shape

## Publish Service Crate

Implemented in:

```text
crates/publish-service/
```

Current behavior:

- Wraps publish dry-run preview behind request/response structs
- Builds planned publication history records
- Preserves blocked dry-run state
- Exposes preview and planned-history generation through `waystone-publishd` D-Bus adapter
- Provides repo-local D-Bus service and systemd user unit activation artifacts
- Does not perform remote mutation

Current tests cover:

- SSH preview and planned-history generation through the service wrapper

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
- Creates feed-entry metadata sidecars under `feeds/entries/` from existing recording metadata
- Validates publication-copy and feed-entry handoff metadata in project context
- Loads audio metadata TOML sidecars
- Lists recording metadata from a directory
- Validates recording IDs
- Validates required title
- Validates project-relative master, published, and feed paths
- Refuses metadata sidecar paths that escape the project root
- Refuses to overwrite existing metadata and feed-entry sidecars
- Generates minimal Atom feed XML from validated `feeds/entries/*.toml` sidecars
- Validates positive channel count and sample rate when present
- Warns on unusual MIME type shape
- Does not copy audio files
- Does not merge existing feed XML or generate non-Atom feeds
- Does not inspect real audio codecs
- Does not access audio devices

Current tests cover:

- Metadata sidecar creation
- Feed-entry sidecar preparation
- Minimal Atom feed XML generation
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
- Provides a service boundary for attach/prepare-feed-entry/validate-publication/validate-feed-entry/generate-feed/list/inspect/validate
- Exposes list/inspect/validate through `waystone-audiod` D-Bus adapter
- Provides repo-local D-Bus service and systemd user unit activation artifacts
- Does not capture, play audio, merge existing feed XML, or expose feed generation through D-Bus

Current tests cover:

- List and validate through the service wrapper
- Recording metadata attachment through the service wrapper
- Feed-entry metadata preparation through the service wrapper
- Minimal Atom feed XML generation through the service wrapper
- Publication-copy and feed-entry handoff validation through the service wrapper

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
record prepare-feed-entry [--json] PROJECT RECORDING_ID UPDATED SUMMARY
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

`record prepare-feed-entry` creates a feed-entry metadata sidecar under
`feeds/entries/` for an existing recording sidecar. It requires the recording
sidecar to include published audio and publication fields and requires the
published audio file to exist. It does not generate or update feed XML.

`record validate-publication` validates an existing recording sidecar in project
context, including referenced master and publication-copy files. `record
validate-feed-entry` validates a prepared feed-entry sidecar against its
recording metadata, enclosure file, and sibling feed-entry IDs.

`record generate-feed` reads the selected project's enabled Atom `[feed]`
configuration, validates prepared feed-entry sidecars, and atomically writes the
configured feed XML path. It does not merge existing feed XML, support RSS, copy
audio, transcode audio, or publish remotely.

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
- Uses `record attach --json` in the Create pane to create audio metadata sidecars for existing project-local master and publication-copy files when the selected project has audio metadata configured
- Uses `record prepare-feed-entry --json`, `record validate-publication --json`, and `record validate-feed-entry --json` in the Create pane to prepare and validate local feed-entry handoff metadata for attached recordings
- Uses `record generate-feed --json` in the Create pane to generate local Atom feed XML from prepared feed-entry sidecars
- Uses configured local projects plus existing `publish` CLI JSON output for read-only Publish-pane dry-run previews
- Derives Publish-pane target choices from `project inspect --json` instead of hard-coded project IDs
- Shows Publish-pane preview status as ready, blocked, failed, no project, or no target
- Filters visible Publish-pane projects by project name, ID, type, path, or target names
- Shows a compact per-target Publish-pane overview for the selected project using read-only dry-run status, method, upload count, verification count, and destination
- Lets Publish-pane target overview row selection update the active target and refresh the existing preview/history panes
- Shows planned publication history file-action summary and raw TOML generated by `publish --planned-history --json`
- Saves planned history previews under the selected project `history/previews/` directory through `publish --save-planned-history-preview --json`
- Lists saved planned history previews for the selected project through `publish --list-planned-history-previews --json`
- Loads selected saved planned history preview TOML through `publish --read-planned-history-preview --json`
- Preserves the selected saved preview row across refreshes when the preview still exists
- Compares generated planned-history TOML against the selected saved preview and reports the first differing line
- Filters saved preview records by filename or path inside the already loaded project-local preview list
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
- Provides diagnostic project create/save, recording attachment/feed-entry/feed-generation, and Publish-pane target/status smoke modes for temporary workspace roots
- Uses static placeholder resource data for Explore
- Mutates only persistent user root settings, minimal project directories under the configured projects root, removable publish target metadata, selected local project content index files, planned preview records, selected project audio metadata sidecars, selected project feed-entry sidecars, and selected project feed XML
- Does not call Rust crates directly, D-Bus, sibling apps, audio devices, or remote services

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
- Implements `PreviewPublication` and `BuildPlannedHistory`
- Requests single-owner bus name behavior; duplicate daemon instances fail quickly
- Provides repo-local D-Bus service and systemd user unit activation artifacts
- D-Bus autostart is smoke-tested through a generated temporary service file
- Systemd user unit syntax is smoke-tested through a generated temporary daemon path
- Does not compare remote state, perform transfers, delete remote files, unlock credentials, or verify remote results
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
- Implements `ListRecordings`, `InspectRecording`, and `ValidateRecording`
- Requests single-owner bus name behavior; duplicate daemon instances fail quickly
- Provides repo-local D-Bus service and systemd user unit activation artifacts
- D-Bus autostart is smoke-tested through a generated temporary service file
- Systemd user unit syntax is smoke-tested through a generated temporary daemon path
- Audio capture not implemented
- Uses `crates/audio-service/` as its internal boundary

## CLI Integration Tests

Implemented in each CLI crate under `cli/*/tests/`.

Current tests cover:

- `project validate` reports invalid fixtures
- `publish --dry-run --json` reports resolved host and identity metadata
- `host validate` rejects invalid trust state
- `identity validate` rejects private-key material
- `record validate` rejects invalid audio paths
- `record attach --json` creates recording metadata, `record prepare-feed-entry --json` creates feed-entry metadata, `record validate-publication --json` plus `record validate-feed-entry --json` validate local audio publication handoff, and `record generate-feed --json` writes a local Atom feed
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
```

Local result on 2026-07-19: Qt 6 was discoverable after installing `qt6-base-dev`; configure and build passed. The offscreen Qt startup smoke script launched the app successfully and verified root handling. The focused Qt project smoke created a minimal project in a generated `/tmp` workspace root, added a removable `export` target, loaded its content index, saved edited Gemtext through the Qt CLI adapter, verified the Create pane content-file list reported `index.gmi`, verified the content-file filter isolated `index.gmi`, verified selected-file detail for index and non-index content files, verified `project inspect --json` reported the target, validated the result, and verified a removable publish dry-run preview. The same smoke creates an audio-capable temporary project, verifies that project creation supplies audio/feed scaffold defaults, verifies that the Create-pane recording attachment controls create an inspectable metadata sidecar for existing project-local audio files, prepares a feed-entry sidecar, verifies publication/feed-entry validation status, generates feed XML, and verifies that the generated Atom feed contains the expected entry. The same focused smoke also creates a separate temporary project with multiple publish targets and verifies that the Publish pane target selector drives ready, blocked, project filtering, per-target overview rows, overview-row target selection, planned-history summary, raw planned-history record preview, saved planned-history preview transitions, saved-preview listing, selected saved-preview detail loading, saved-preview row selection preservation, generated-vs-saved comparison reporting, and saved-preview filtering. The Publish pane derives all discovered project targets into a selector, filters visible projects, reports dry-run preview state as ready, blocked, failed, no project, or no target, shows a compact per-target overview, lets overview row selection update the active target, shows planned publication history file-action grouping plus raw TOML, saves planned previews under project `history/previews/`, lists saved preview records, loads selected preview TOML, preserves the selected saved preview across refreshes, reports first-line differences between generated and selected saved planned history, and filters the visible saved-preview list without writing completed history or mutating remotes. The CLI JSON contract smoke verifies `record attach --json`, `record prepare-feed-entry --json`, `record validate-publication --json`, `record validate-feed-entry --json`, and `record generate-feed --json` against temporary project data. The projectd D-Bus smoke script verified create, list, inspect, validate, invalid-request handling, unavailable-bus failure, and duplicate-owner failure on a private test session bus. The publishd D-Bus smoke script verified preview, planned-history generation, invalid-request handling, unavailable-bus failure, and duplicate-owner failure on a private test session bus. The host/identity D-Bus smoke script verified list, inspect, validate, invalid-request handling, unavailable-bus failure, and duplicate-owner failure for both adapters on a private test session bus. The audiod D-Bus smoke script verified list, inspect, validate, invalid-request handling, unavailable-bus failure, and duplicate-owner failure on a private test session bus. The activation smokes verified projectd, publishd, host/identity, and audiod D-Bus service-file autostart. The systemd smokes verified projectd, publishd, host/identity, and audiod unit syntax through temporary paths.

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
- Audio trimming, normalization, or export
- Audio metadata replacement or merge editing
- Existing feed XML merge updates or non-Atom feed generation
- Deeper Workspace actions beyond local inspect, authoring, preview, and feed generation
- Live reload after editing persistent user settings
- Browser, Helm, or Comm integration
