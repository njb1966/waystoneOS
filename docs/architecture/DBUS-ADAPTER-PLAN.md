# WaystoneOS D-Bus Adapter Plan

Status: Project, publish, host, identity, and audio adapters active
Date: 2026-07-18

This document defines the first D-Bus adapter work. It intentionally does not add new domain behavior, new persistent formats, remote mutation, credential unlock, or GUI integration.

## Decision Boundary

The first D-Bus implementation slice should adapt the existing service crate contract for `waystone-projectd`.

Reasons:

- Project operations are foundational for publishing, audio attachment, and Workspace views.
- `services/projectd` already exists as a daemon scaffold.
- `crates/project-service` already exposes narrow request and response structs.
- Current operations are local and inspectable.
- The initial method set can avoid credentials, remotes, audio devices, and destructive filesystem changes beyond the already implemented create operation.

## Implemented Adapter Scope

Project daemon:

```text
services/projectd
```

Target interface:

```text
org.waystone.Project1
/org/waystone/Project
```

Implemented methods:

```text
CreateProject
ListProjects
InspectProject
ValidateProject
```

Still deferred:

- Project migration, repair, archive, and export
- GUI migration from CLI adapter to D-Bus
- Authorization prompts
- Cross-service calls

`CreateProject` mutates the filesystem, but only through the existing `ProjectService` behavior and caller-supplied parent path. It must continue to refuse overwrites and invalid project IDs through the service crate.

Host daemon:

```text
services/hostd
```

Target interface:

```text
org.waystone.Host1
/org/waystone/Host
```

Implemented methods:

```text
ListHosts
InspectHost
ValidateHost
```

Identity daemon:

```text
services/identityd
```

Target interface:

```text
org.waystone.Identity1
/org/waystone/Identity
```

Implemented methods:

```text
ListIdentities
InspectIdentity
ValidateIdentity
```

Still deferred for host and identity services:

- Host creation, update, removal, and trust mutation
- Identity creation, import, export, removal, lock, and credential unlock
- SSH host-key probing
- Secret storage
- Installing activation files into user or system service directories

Publish daemon:

```text
services/publishd
```

Target interface:

```text
org.waystone.Publish1
/org/waystone/Publish
```

Implemented methods:

```text
PreviewPublication
BuildPlannedHistory
```

Still deferred for publish service:

- Remote comparison
- Remote transfer execution
- Remote deletion
- Remote verification
- Credential unlock
- Writing completed history records
- Installing activation files into user or system service directories

Audio daemon:

```text
services/audiod
```

Target interface:

```text
org.waystone.Audio1
/org/waystone/Audio
```

Implemented methods:

```text
ListRecordings
InspectRecording
ValidateRecording
```

Still deferred for audio service:

- Audio capture
- Audio playback
- Audio device enumeration
- Audio editing, trimming, normalization, and codec export
- Installing activation files into user or system service directories

## Adapter Rules

- D-Bus methods adapt existing service crate operations.
- The service crate remains the owner of domain behavior.
- The daemon owns request dispatch, IPC error mapping, process lifecycle, and logging.
- The daemon must request its well-known bus name as a single owner; duplicate instances must fail instead of replacing or queueing.
- D-Bus payloads should stay schema-versioned and structured.
- Errors must remain secret-safe and suitable for CLI or GUI display.
- No adapter method may read outside caller-supplied roots except through the service crate's existing validation rules.
- No remote system may be contacted.

## Payload Shape

The first implementation should prefer a simple JSON string payload over many ad hoc D-Bus scalar arguments.

Example request:

```json
{
  "schema": 1,
  "root": "/workspace/Projects"
}
```

Example response:

```json
{
  "schema": 1,
  "ok": true,
  "data": {
    "projects": []
  }
}
```

Example error:

```json
{
  "schema": 1,
  "ok": false,
  "error": {
    "code": "invalid_request",
    "message": "Project root is missing"
  }
}
```

This keeps the first adapter close to the existing CLI JSON contract while avoiding premature D-Bus type design. Typed D-Bus structs can be introduced later if the interface stabilizes and clients need stronger introspection.

## Rust Implementation Shape

Expected adapter structure:

```text
services/projectd/
  src/main.rs
  src/dbus.rs
  tests/
services/publishd/
  src/main.rs
  src/dbus.rs
services/hostd/
  src/main.rs
  src/dbus.rs
services/identityd/
  src/main.rs
  src/dbus.rs
services/audiod/
  src/main.rs
  src/dbus.rs
```

Current crate direction:

- Add D-Bus dependencies only to daemon crates that expose IPC.
- Keep `crates/project-service` dependency-free from D-Bus.
- Keep `crates/publish-service` dependency-free from D-Bus.
- Keep `crates/host-service` and `crates/identity-service` dependency-free from D-Bus.
- Keep `crates/audio-service` dependency-free from D-Bus.
- Use the daemon binary for the session bus service.
- Add integration tests that launch the daemon against a temporary session bus when practical.

Selected dependency:

```text
zbus 5.13.1
```

`zbus` was selected because it is Rust-native, MIT-licensed, supports service-side D-Bus interfaces, and has a blocking API suitable for the first daemon slice. The dependency is pinned to the newest checked `zbus` release compatible with the currently installed Rust 1.85.0 toolchain. Newer `zbus` releases currently require Rust 1.87.

## Verification Gates

The first implementation slice should pass:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
scripts/cli-json-contract-smoke.sh
scripts/workspace-qt-smoke.sh
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

Additional D-Bus verification should prove:

- `waystone-projectd` can start and own `org.waystone.Project1` on a test session bus.
- `CreateProject` creates a valid minimal project in a caller-supplied temporary parent.
- `ListProjects` returns expected project IDs for repository examples.
- `InspectProject` returns expected core identity fields.
- `ValidateProject` reports invalid fixtures without panicking.
- Invalid JSON requests return a structured `invalid_request` response.
- The daemon reports startup failure cleanly when a session bus is unavailable.
- A duplicate daemon instance on the same session bus fails quickly instead of taking over the name.
- D-Bus service-file autostart works on a private test session bus using a generated temporary service file.
- The checked-in systemd user unit verifies after substituting a temporary daemon path.
- `waystone-publishd` can start, own `org.waystone.Publish1`, reject duplicate ownership, and serve preview, planned-history, and invalid-request responses.
- Publish D-Bus service-file autostart works on a private test session bus using a generated temporary service file.
- The checked-in publish systemd user unit verifies after substituting a temporary daemon path.
- `waystone-hostd` can start, own `org.waystone.Host1`, reject duplicate ownership, and serve list, inspect, validate, and invalid-request responses.
- `waystone-identityd` can start, own `org.waystone.Identity1`, reject duplicate ownership, and serve list, inspect, validate, and invalid-request responses.
- Host and identity D-Bus service-file autostart works on a private test session bus using generated temporary service files.
- The checked-in host and identity systemd user units verify after substituting temporary daemon paths.
- `waystone-audiod` can start, own `org.waystone.Audio1`, reject duplicate ownership, and serve list, inspect, validate, and invalid-request responses.
- Audio D-Bus service-file autostart works on a private test session bus using a generated temporary service file.
- The checked-in audio systemd user unit verifies after substituting a temporary daemon path.

## Non-Goals

- Do not replace the Qt Workspace CLI adapter in the same slice.
- Do not add remote publication, credential unlock, or host-key probing.
- Do not add audio capture, playback, device enumeration, or editing.
- Do not add files outside this repository.

## Next Work

1. Add focused smoke coverage for Publish-pane target/status behavior, preferably through a diagnostic mode that exercises multiple target metadata without remote publication.
2. Continue the Publish workflow toward planned publication history preview.
3. Keep Qt Workspace on CLI adapters until D-Bus activation behavior is stable in installed environments.
4. Add install/package automation only when the repo has a broader install layout.
5. Keep project migration, repair, archive, export, credential unlock, host-key probing, remote transfer, and cross-service calls deferred.
