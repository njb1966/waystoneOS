# WaystoneOS Service Contracts

Status: Current implementation contract
Date: 2026-07-20

This document records the service contracts that exist now and the D-Bus names they map to or are expected to map to later.

The current implementation uses Rust crates with request and response structs. `waystone-projectd` exposes project creation, listing, inspection, and validation over D-Bus. `waystone-publishd` exposes publication preview, publication readiness validation, read-only transfer-intent reporting, planned-history generation, and completed-history result-record generation/save/list/read over D-Bus. `waystone-hostd` and `waystone-identityd` expose read-only list, inspect, and validate operations over D-Bus. `waystone-audiod` exposes recording list, inspect, validate, local sidecar attachment/update, WAV master capture from explicit `ffmpeg` input sources, Opus publication-copy export, feed-entry sidecar preparation/update, publication/feed-entry handoff validation, and local Atom feed generation over D-Bus. These five daemons have repo-local activation artifacts. No activation files are installed outside this repository.

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
| Publishing | `crates/publish-service` | `services/publishd` | `org.waystone.Publish1` | preview dry-run, publication readiness validation, transfer-intent reporting, removable execution preparation, planned history, completed-history record construction/save/list/read; D-Bus adapter for preview, validation, transfer-intent reporting, planned history, and completed-history generation/save/list/read |
| Hosts | `crates/host-service` | `services/hostd` | `org.waystone.Host1` | list, inspect, validate; D-Bus adapter for list, inspect, validate |
| Identities | `crates/identity-service` | `services/identityd` | `org.waystone.Identity1` | list, inspect, validate; D-Bus adapter for list, inspect, validate |
| Audio metadata | `crates/audio-service` | `services/audiod` | `org.waystone.Audio1` | attach, update, WAV master capture, Opus publication-copy export, prepare/update feed entry, validate publication, validate feed entry, generate feed, list, inspect, validate; D-Bus adapter for all listed operations |

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
- `ValidatePublicationRequest`
- `ValidatePublicationResponse`
- `TransferIntentRequest`
- `TransferIntentResponse`
- `PrepareRemovableExecutionRequest`
- `PrepareRemovableExecutionResponse`
- `BuildPlannedHistoryRequest`
- `BuildPlannedHistoryResponse`
- `BuildCompletedHistoryRequest`
- `BuildCompletedHistoryResponse`
- `SaveCompletedHistoryRequest`
- `SaveCompletedHistoryResponse`
- `ListCompletedHistoryRequest`
- `ListCompletedHistoryResponse`
- `ReadCompletedHistoryRequest`
- `ReadCompletedHistoryResponse`
- `PublishService`

Current behavior:

- Builds non-mutating dry-run publication plans through `waystone-publish-plan`.
- Reports feed readiness in dry-run previews, including configured path, file existence, prepared entry count, invalid entry count, and per-invalid-sidecar diagnostics.
- Builds non-mutating publication readiness reports with stable issue codes for project validation, host/identity resolution, enabled-feed readiness, invalid feed-entry sidecars, empty file plans, and required confirmations.
- Builds non-mutating transfer-intent reports from immediate validation plus
  dry-run state.
- Reports `execution_ready`, blocking issues, confirmations, host/identity
  resolution summaries, comparison metadata, change buckets, and the future
  completed-history directory without accessing credentials or transferring.
- Builds non-mutating removable executor preparation plans with a bounded local
  destination root and per-file source/destination operation records.
- Blocks removable execution preparation for unsupported methods, existing
  transfer-intent blockers, and delete operations.
- Resolves host and identity metadata when caller supplies roots.
- Accepts caller-supplied local remote-state manifests for dry-run comparison.
- Builds planned and completed publication history records through `waystone-publication-history`.
- Saves, lists, and reads completed history records under project `history/completed/`.
- Preserves blocked dry-run state.
- Exposes preview, publication readiness validation, read-only transfer-intent
  reporting, planned-history generation, and completed-history result-record
  generation/save/list/read through `waystone-publishd` D-Bus adapter.
- Does not expose removable execution preparation through D-Bus yet.
- Does not probe remote hosts, transfer files, execute deletions, verify remotes, or unlock credentials.

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
- `CaptureRecordingRequest`
- `ExportOpusRequest`
- `PrepareFeedEntryRequest`
- `UpdateFeedEntryRequest`
- `UpdateRecordingRequest`
- `ValidatePublicationRequest`
- `ValidateFeedEntryRequest`
- `GenerateFeedRequest`
- `AudioService`

Current behavior:

- Creates recording metadata sidecars for existing project-local master and publication-copy files.
- Updates existing recording metadata sidecars while preserving sidecar path, embedded recording ID, and optional measurement fields.
- Captures mono 48 kHz PCM WAV masters under the configured project `[audio].masters` root from explicit local `ffmpeg` input sources.
- Creates Opus publication-copy files from existing project-local masters through `ffmpeg/libopus`.
- Creates feed-entry sidecars from existing recording metadata and published audio references.
- Updates existing feed-entry sidecars from current recording metadata and command-provided update fields.
- Validates publication-copy and feed-entry handoff metadata in project context.
- Generates minimal Atom feed XML from validated feed-entry sidecars through the local service crate, replacing matching existing Atom entries by ID and preserving unrelated entries.
- Lists recording sidecar metadata.
- Loads and inspects recording metadata.
- Validates project-relative audio and feed paths.
- Refuses to overwrite existing sidecars.
- Refuses to overwrite captured masters and publication-copy outputs.
- Exposes the local audio/feed operations above through `waystone-audiod` D-Bus using JSON string payloads that adapt the service crate request/response structs.
- Does not enumerate audio devices, play audio, edit audio, perform codec transcoding beyond Opus publication export, or merge remote feed state.

## D-Bus Mapping Notes

When D-Bus is added, each interface should start from the current service crate operations and expose stable method names with schema-versioned payloads.

Initial mapping convention:

```text
org.waystone.Project1.CreateProject
org.waystone.Project1.ListProjects
org.waystone.Project1.InspectProject
org.waystone.Project1.ValidateProject

org.waystone.Publish1.PreviewPublication
org.waystone.Publish1.ValidatePublication
org.waystone.Publish1.TransferIntent
org.waystone.Publish1.BuildPlannedHistory
org.waystone.Publish1.BuildCompletedHistory
org.waystone.Publish1.SaveCompletedHistory
org.waystone.Publish1.ListCompletedHistory
org.waystone.Publish1.ReadCompletedHistory

org.waystone.Host1.ListHosts
org.waystone.Host1.InspectHost
org.waystone.Host1.ValidateHost

org.waystone.Identity1.ListIdentities
org.waystone.Identity1.InspectIdentity
org.waystone.Identity1.ValidateIdentity

org.waystone.Audio1.ListRecordings
org.waystone.Audio1.InspectRecording
org.waystone.Audio1.ValidateRecording
org.waystone.Audio1.AttachRecording
org.waystone.Audio1.UpdateRecording
org.waystone.Audio1.CaptureRecording
org.waystone.Audio1.ExportOpus
org.waystone.Audio1.PrepareFeedEntry
org.waystone.Audio1.UpdateFeedEntry
org.waystone.Audio1.ValidatePublication
org.waystone.Audio1.ValidateFeedEntry
org.waystone.Audio1.GenerateFeed
```

The first D-Bus slices are adapter work only: daemon lifecycle, method dispatch, error mapping, and integration tests. They do not change persistent formats or add remote mutation as a side effect.

The accepted D-Bus direction is documented in [DBUS-ADAPTER-PLAN.md](DBUS-ADAPTER-PLAN.md) and [ADR-0013](../decisions/0013-first-dbus-adapter-slice.md): adapt service crates through narrow daemon interfaces, keep domain behavior out of IPC code, and defer GUI migration until activation and install behavior are stable.
