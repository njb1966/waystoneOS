# WaystoneOS D-Bus Adapter Plan

Status: Project create adapter active
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

## First Adapter Scope

Target daemon:

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
- Installing activation files into user or system service directories
- Authorization prompts
- Cross-service calls

`CreateProject` mutates the filesystem, but only through the existing `ProjectService` behavior and caller-supplied parent path. It must continue to refuse overwrites and invalid project IDs through the service crate.

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

Expected first-pass structure:

```text
services/projectd/
  src/main.rs
  src/dbus.rs
  tests/
```

Current crate direction:

- Add D-Bus dependencies only to `services/projectd`.
- Keep `crates/project-service` dependency-free from D-Bus.
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
scripts/projectd-dbus-activation-smoke.sh
scripts/projectd-systemd-unit-smoke.sh
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

## Non-Goals

- Do not replace the Qt Workspace CLI adapter in the same slice.
- Do not add host, identity, audio, or publish D-Bus adapters until the project adapter pattern is working.
- Do not add remote publication, credential unlock, or host-key probing.
- Do not add files outside this repository.

## Next Work

1. Decide whether to add host and identity D-Bus adapters next.
2. Keep Qt Workspace on CLI adapters until D-Bus activation behavior is stable in installed environments.
3. Add install/package automation only when the repo has a broader packaging layout.
4. Keep project migration, repair, archive, export, and cross-service calls deferred.
