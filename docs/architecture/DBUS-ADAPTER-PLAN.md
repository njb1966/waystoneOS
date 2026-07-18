# WaystoneOS D-Bus Adapter Plan

Status: Draft for next implementation slice
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

Initial methods:

```text
ListProjects
InspectProject
ValidateProject
```

Deferred from the first D-Bus pass:

- `CreateProject`
- Project migration, repair, archive, and export
- GUI migration from CLI adapter to D-Bus
- D-Bus activation files
- systemd user units
- Authorization prompts
- Cross-service calls

`CreateProject` already exists in the service crate, but it mutates the filesystem. It should be added only after the read-only adapter path, error mapping, and integration test pattern are stable.

## Adapter Rules

- D-Bus methods adapt existing service crate operations.
- The service crate remains the owner of domain behavior.
- The daemon owns request dispatch, IPC error mapping, process lifecycle, and logging.
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

Likely crate direction:

- Add a D-Bus crate only to `services/projectd`.
- Keep `crates/project-service` dependency-free from D-Bus.
- Use the daemon binary for the session bus service.
- Add integration tests that launch the daemon against a temporary session bus when practical.

The preferred implementation dependency should be chosen immediately before implementation based on current Debian stable packaging and Rust ecosystem fit. The likely candidate is `zbus`, but this plan does not commit the repo to that dependency yet.

## Verification Gates

The first implementation slice should pass:

```bash
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
scripts/cli-json-contract-smoke.sh
scripts/workspace-qt-smoke.sh
```

Additional D-Bus verification should prove:

- `waystone-projectd` can start and own `org.waystone.Project1` on a test session bus.
- `ListProjects` returns the same project IDs as the `project list --json` CLI for repository examples.
- `InspectProject` returns the same core identity fields as the CLI inspect path.
- `ValidateProject` reports invalid fixtures without panicking or leaking host paths beyond expected user-supplied paths.
- The daemon exits cleanly when the test bus is unavailable or closed.

## Non-Goals

- Do not replace the Qt Workspace CLI adapter in the same slice.
- Do not add project create over D-Bus in the same slice.
- Do not add host, identity, audio, or publish D-Bus adapters until the project adapter pattern is working.
- Do not add remote publication, credential unlock, or host-key probing.
- Do not add files outside this repository.

## Next Work

1. Select and justify the Rust D-Bus dependency for `services/projectd`.
2. Implement read-only `waystone-projectd` D-Bus methods.
3. Add a test helper or script for exercising the daemon on a test session bus.
4. Update service contracts after the adapter behavior is verified.
