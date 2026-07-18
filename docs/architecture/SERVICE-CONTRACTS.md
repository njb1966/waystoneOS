# WaystoneOS Service Contracts

Status: Current implementation contract
Date: 2026-07-17

This document records the service contracts that exist now and the D-Bus names they are expected to map to later.

The current implementation uses Rust crates with request and response structs. `waystone-projectd` has the first read-only D-Bus adapter for project listing, inspection, and validation. The other daemon binaries are placeholders. No D-Bus activation, authorization layer, or systemd integration exists yet.

## Contract Rules

- Domain behavior lives in reusable crates.
- CLI and future GUI code should call service crates or the same command semantics, not duplicate domain behavior.
- Placeholder daemons may reference service crates, but they must not invent separate behavior.
- Future D-Bus interfaces adapt the service crate contracts; they do not own the business rules.
- Request and response types should remain narrow, explicit, and serializable in shape even before IPC is implemented.
- Errors must stay secret-safe and suitable for human or JSON CLI output.
- Remote mutation, credential unlock, and destructive operations require explicit contract additions before implementation.

## Current Contracts

| Domain | Current crate | Placeholder daemon | Future D-Bus name | Current operations |
| --- | --- | --- | --- | --- |
| Projects | `crates/project-service` | `services/projectd` | `org.waystone.Project1` | create, list, inspect, validate; D-Bus adapter for list, inspect, validate |
| Publishing | `crates/publish-service` | not scaffolded yet | `org.waystone.Publish1` | preview dry-run, planned history |
| Hosts | `crates/host-service` | `services/hostd` | `org.waystone.Host1` | list, inspect, validate |
| Identities | `crates/identity-service` | `services/identityd` | `org.waystone.Identity1` | list, inspect, validate |
| Audio metadata | `crates/audio-service` | `services/audiod` | `org.waystone.Audio1` | list, inspect, validate |

## Project Service

Current Rust crate:

```text
crates/project-service/
```

Future D-Bus interface:

```text
org.waystone.Project1
```

Current contract:

- `CreateProjectRequest`
- `ListProjectsRequest`
- `InspectProjectRequest`
- `ValidateProjectRequest`
- `ProjectService`

Current behavior:

- Creates minimal `.wayproject` directories through `waystone-project-format`.
- Lists project manifests below a root.
- Inspects project identity, schema, type, and content paths.
- Validates project manifests and content entrypoints.
- Does not migrate, repair, archive, export, or maintain a daemon cache.

## Publish Service

Current Rust crate:

```text
crates/publish-service/
```

Future D-Bus interface:

```text
org.waystone.Publish1
```

Current contract:

- `PreviewPublicationRequest`
- `PreviewPublicationResponse`
- `BuildPlannedHistoryRequest`
- `BuildPlannedHistoryResponse`
- `PublishService`

Current behavior:

- Builds non-mutating dry-run publication plans through `waystone-publish-plan`.
- Resolves host and identity metadata when caller supplies roots.
- Builds planned publication history records through `waystone-publication-history`.
- Preserves blocked dry-run state.
- Does not compare remote state, transfer files, delete files, verify remotes, or unlock credentials.

## Host Service

Current Rust crate:

```text
crates/host-service/
```

Future D-Bus interface:

```text
org.waystone.Host1
```

Current contract:

- `ListHostsRequest`
- `InspectHostRequest`
- `ValidateHostRequest`
- `HostService`

Current behavior:

- Lists host metadata from a caller-supplied root.
- Loads and inspects host records.
- Validates IDs and trust states.
- Does not probe networks, inspect SSH host keys, or write trust changes.

## Identity Service

Current Rust crate:

```text
crates/identity-service/
```

Future D-Bus interface:

```text
org.waystone.Identity1
```

Current contract:

- `ListIdentitiesRequest`
- `InspectIdentityRequest`
- `ValidateIdentityRequest`
- `IdentityService`

Current behavior:

- Lists identity metadata from a caller-supplied root.
- Loads and inspects identity records.
- Validates public key and certificate metadata.
- Detects private-key material markers in identity TOML.
- Does not unlock, generate, import, export, revoke, or store credentials.

## Audio Service

Current Rust crate:

```text
crates/audio-service/
```

Future D-Bus interface:

```text
org.waystone.Audio1
```

Current contract:

- `ListRecordingsRequest`
- `InspectRecordingRequest`
- `ValidateRecordingRequest`
- `AudioService`

Current behavior:

- Lists recording sidecar metadata.
- Loads and inspects recording metadata.
- Validates project-relative audio and feed paths.
- Does not enumerate audio devices, capture audio, play audio, edit audio, or export codecs.

## D-Bus Mapping Notes

When D-Bus is added, each interface should start from the current service crate operations and expose stable method names with schema-versioned payloads.

Initial mapping convention:

```text
org.waystone.Project1.CreateProject
org.waystone.Project1.ListProjects
org.waystone.Project1.InspectProject
org.waystone.Project1.ValidateProject

org.waystone.Publish1.PreviewPublication

org.waystone.Host1.ListHosts
org.waystone.Host1.InspectHost
org.waystone.Host1.ValidateHost

org.waystone.Identity1.ListIdentities
org.waystone.Identity1.InspectIdentity
org.waystone.Identity1.ValidateIdentity

org.waystone.Audio1.ListRecordings
org.waystone.Audio1.InspectRecording
org.waystone.Audio1.ValidateRecording
```

The first D-Bus slice should be adapter work only: daemon lifecycle, method dispatch, error mapping, and integration tests. It should not change persistent formats or add remote mutation as a side effect.

The accepted first slice is documented in [DBUS-ADAPTER-PLAN.md](DBUS-ADAPTER-PLAN.md) and [ADR-0013](../decisions/0013-first-dbus-adapter-slice.md): start with read-only `waystone-projectd` methods before adding mutating project operations or other daemons.
