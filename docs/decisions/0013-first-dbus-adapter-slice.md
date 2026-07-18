# ADR-0013: First D-Bus Adapter Slice

Status: Accepted
Date: 2026-07-18

## Context

WaystoneOS already has service crates and placeholder daemon binaries for projects, hosts, identities, and audio metadata. The Qt Workspace currently talks to CLI JSON output and remains local-only.

The project needs a first D-Bus adapter path, but the adapter must not accidentally expand scope into GUI rewiring, remote mutation, credential handling, systemd activation, or new persistent formats.

## Decision

The first D-Bus adapter slice will target `waystone-projectd` only.

The first method set will be read-only:

- `ListProjects`
- `InspectProject`
- `ValidateProject`

The adapter will map D-Bus calls to the existing `crates/project-service` contract. The service crate remains free of D-Bus dependencies and remains the owner of project behavior.

The first payload shape may use schema-versioned JSON strings over D-Bus. This keeps the adapter close to the existing CLI JSON contract while the service boundary stabilizes.

`services/projectd` will use `zbus` for the first adapter. The dependency is pinned to `zbus` 5.13.1 because it is the newest checked `zbus` release compatible with the current Rust 1.85.0 toolchain. Newer checked releases require Rust 1.87.

## Consequences

- The first IPC implementation exercises daemon lifecycle, bus ownership, method dispatch, and error mapping without adding filesystem mutation.
- `CreateProject` remains available through existing code paths but is deferred from D-Bus until the read-only adapter is proven.
- Host, identity, audio, and publish daemons wait for the project adapter pattern.
- The Qt Workspace continues to use CLI adapters until D-Bus behavior and tests are stable.
- D-Bus typed structs may replace or supplement JSON payloads later if client needs justify the complexity.

## Alternatives Considered

Implement every existing daemon at once:

- Rejected because it would multiply dependency, lifecycle, and testing questions before one adapter pattern exists.

Start with `waystone-publishd`:

- Rejected because publishing has remote-transfer and credential-adjacent implications even when limited to dry-run planning.

Start with host or identity services:

- Rejected because those domains are closer to trust and credential workflows. Their D-Bus adapters should reuse lessons from a lower-risk project adapter.

Immediately migrate Qt Workspace from CLI to D-Bus:

- Rejected because it would mix service-adapter work with UI state ownership changes.

## Follow-Up

- Decide whether `CreateProject` should be the first mutating D-Bus method.
- Keep activation and systemd user units deferred until direct daemon behavior is stable.
- Keep Qt Workspace on CLI adapters until D-Bus lifecycle and error behavior are stable.
