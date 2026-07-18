# WaystoneOS Implementation Status

Status: Current as of 2026-07-18

This file records what exists in the repository now. It should be updated whenever a planning contract becomes implementation.

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
- Lists projects with bounded category-depth discovery

Current tests cover:

- Valid minimal capsule fixture
- Missing content index
- Path traversal rejection
- Absolute path rejection
- Project inspection
- Minimal project creation
- Invalid project ID rejection
- Bounded project listing

## Project CLI

Implemented in:

```text
cli/project/
```

Current commands:

```text
project create [--json] PARENT ID NAME TYPE
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
- Does not implement D-Bus activation

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
- Does not implement D-Bus activation

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
```

Current behavior:

- Human-readable dry-run transfer plan
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

- Loads audio metadata TOML sidecars
- Lists recording metadata from a directory
- Validates recording IDs
- Validates required title
- Validates project-relative master, published, and feed paths
- Validates positive channel count and sample rate when present
- Warns on unusual MIME type shape
- Does not inspect real audio codecs
- Does not access audio devices

Current tests cover:

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
- Provides a service boundary for list/inspect/validate
- Does not implement D-Bus activation
- Does not capture or play audio

Current tests cover:

- List and validate through the service wrapper

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
record list [--json] ROOT
record inspect [--json] PATH
record validate [--json] PATH
```

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
- Lists current placeholder service binaries
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
- Uses the existing `project` CLI JSON output for read-only Create-pane project list, inspect, and validate
- Uses the existing `record` and `listen` CLI JSON output for read-only Create-pane recording list, inspect, validate, and playable state
- Uses the existing `publish` CLI JSON output for read-only Publish-pane dry-run previews
- Uses the existing `host` and `identity` CLI JSON output for read-only Operate-pane list, inspect, and validate
- Keeps command execution and JSON parsing in `ui/workspace-qt/src/cli_adapter.*`
- Keeps page construction in `ui/workspace-qt/src/workspace_pages.*`
- Keeps local Workspace root configuration in `ui/workspace-qt/src/workspace_config.*`
- Provides example root configuration in `ui/workspace-qt/workspace.example.ini`
- Shows active configured roots in the Explore pane
- Uses static placeholder resource data for Explore
- Does not call Rust crates directly, D-Bus, sibling apps, audio devices, or remote services

## Project Service

Implemented in:

```text
services/projectd/
```

Current state:

- Placeholder binary only
- D-Bus service not implemented
- Service contract documented in `docs/architecture/PROJECT-SERVICE.md`
- Uses `crates/project-service/` as its internal boundary

## Host and Identity Services

Implemented in:

```text
services/hostd/
services/identityd/
services/audiod/
```

Current state:

- Placeholder binaries only
- D-Bus services not implemented
- Metadata logic remains in `crates/host-identity/`
- `hostd` uses `crates/host-service/` as its internal boundary
- `identityd` uses `crates/identity-service/` as its internal boundary

## Audio Service

Implemented in:

```text
services/audiod/
```

Current state:

- Placeholder binary only
- D-Bus service not implemented
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
```

Local result on 2026-07-18: Qt 6 was discoverable after installing `qt6-base-dev`; configure and build passed. The offscreen Qt startup smoke script launched the app successfully and confirmed that it remained in the Qt event loop until timeout.

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
```

## Not Implemented Yet

- D-Bus activation
- Long-running `waystone-projectd`
- Long-running `waystone-hostd`
- Long-running `waystone-identityd`
- Long-running `waystone-audiod`
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
- Deeper Workspace actions beyond read-only inspect and preview
- Persistent user settings location outside explicit `--config`
- Browser, Helm, or Comm integration
