# WaystoneOS Service Contracts

Status: Current implementation contract
Date: 2026-07-20

This document records the service contracts that exist now and the D-Bus names they map to or are expected to map to later.

The current implementation uses Rust crates with request and response structs. `waystone-projectd` exposes project creation, listing, inspection, and validation over D-Bus. `waystone-publishd` exposes non-mutating publication preview and planned-history generation over D-Bus. `waystone-hostd`, `waystone-identityd`, and `waystone-audiod` expose read-only list, inspect, and validate operations over D-Bus. The audio service crate also exposes local sidecar attachment, mock publication-copy export, feed-entry sidecar preparation, and publication/feed-entry handoff validation for the CLI, but mutating audio operations are not yet exposed through `waystone-audiod`. These five daemons have repo-local activation artifacts. No activation files are installed outside this repository.

## Contract Rules

- Domain behavior lives in reusable crates.
- CLI and future GUI code should call service crates or the same command semantics, not duplicate domain behavior.
- Daemons may reference service crates, but they must not invent separate behavior.
- D-Bus interfaces adapt the service crate contracts; they do not own the business rules.
- Request and response types should remain narrow, explicit, and serializable in shape even before IPC is implemented.
- Errors must stay secret-safe and suitable for human or JSON CLI output.
- Remote mutation, credential unlock, and destructive operations require explicit contract additions before implementation.

## Current Contracts

| Domain | Current crate | Service daemon | D-Bus name | Current operations |
| --- | --- | --- | --- | --- |
| Projects | `crates/project-service` | `services/projectd` | `org.waystone.Project1` | create, list, inspect, validate; D-Bus adapter for create, list, inspect, validate |
| Publishing | `crates/publish-service` | `services/publishd` | `org.waystone.Publish1` | preview dry-run, planned history; D-Bus adapter for preview and planned history |
| Hosts | `crates/host-service` | `services/hostd` | `org.waystone.Host1` | list, inspect, validate; D-Bus adapter for list, inspect, validate |
| Identities | `crates/identity-service` | `services/identityd` | `org.waystone.Identity1` | list, inspect, validate; D-Bus adapter for list, inspect, validate |
| Audio metadata | `crates/audio-service` | `services/audiod` | `org.waystone.Audio1` | attach, mock Opus publication-copy export, prepare feed entry, validate publication, validate feed entry, generate feed, list, inspect, validate; D-Bus adapter for list, inspect, validate |

## Project Service

Current Rust crate:

```text
crates/project-service/
```

Current D-Bus interface:

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

- Creates `.wayproject` directories through `waystone-project-format`.
- Adds audio/feed scaffold defaults for audio-capable project types.
- Lists project manifests below a root.
- Inspects project identity, schema, type, and content paths.
- Validates project manifests and content entrypoints.
- Does not migrate, repair, archive, export, or maintain a daemon cache.

## Publish Service

Current Rust crate:

```text
crates/publish-service/
```

Current D-Bus interface:

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
- Reports feed readiness in dry-run previews, including configured path, file existence, prepared entry count, invalid entry count, and per-invalid-sidecar diagnostics.
- Resolves host and identity metadata when caller supplies roots.
- Builds planned publication history records through `waystone-publication-history`.
- Preserves blocked dry-run state.
- Exposes preview and planned-history generation through `waystone-publishd` D-Bus adapter.
- Does not compare remote state, transfer files, delete files, verify remotes, or unlock credentials.

## Host Service

Current Rust crate:

```text
crates/host-service/
```

Current D-Bus interface:

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

Current D-Bus interface:

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

Current D-Bus interface:

```text
org.waystone.Audio1
```

Current contract:

- `ListRecordingsRequest`
- `InspectRecordingRequest`
- `ValidateRecordingRequest`
- `AttachRecordingRequest`
- `ExportOpusRequest`
- `PrepareFeedEntryRequest`
- `UpdateRecordingRequest`
- `ValidatePublicationRequest`
- `ValidateFeedEntryRequest`
- `GenerateFeedRequest`
- `AudioService`

Current behavior:

- Creates recording metadata sidecars for existing project-local master and publication-copy files.
- Updates existing recording metadata sidecars while preserving sidecar path, embedded recording ID, and optional measurement fields.
- Creates mock Opus publication-copy files from existing project-local masters for CLI-facing workflow tests.
- Creates feed-entry sidecars from existing recording metadata and published audio references.
- Validates publication-copy and feed-entry handoff metadata in project context.
- Generates minimal Atom feed XML from validated feed-entry sidecars through the local service crate.
- Lists recording sidecar metadata.
- Loads and inspects recording metadata.
- Validates project-relative audio and feed paths.
- Refuses to overwrite existing sidecars.
- Does not enumerate audio devices, capture audio, play audio, edit audio, perform real codec export/transcoding, merge existing feed XML, or expose mutating audio operations over D-Bus.

## D-Bus Mapping Notes

When D-Bus is added, each interface should start from the current service crate operations and expose stable method names with schema-versioned payloads.

Initial mapping convention:

```text
org.waystone.Project1.CreateProject
org.waystone.Project1.ListProjects
org.waystone.Project1.InspectProject
org.waystone.Project1.ValidateProject

org.waystone.Publish1.PreviewPublication
org.waystone.Publish1.BuildPlannedHistory

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

The first D-Bus slices are adapter work only: daemon lifecycle, method dispatch, error mapping, and integration tests. They do not change persistent formats or add remote mutation as a side effect.

The accepted D-Bus direction is documented in [DBUS-ADAPTER-PLAN.md](DBUS-ADAPTER-PLAN.md) and [ADR-0013](../decisions/0013-first-dbus-adapter-slice.md): adapt service crates through narrow daemon interfaces, keep domain behavior out of IPC code, and defer GUI migration until activation and install behavior are stable.
