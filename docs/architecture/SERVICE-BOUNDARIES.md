# WaystoneOS Service Boundaries

Status: Draft for Phase 0
Date: 2026-07-17

WaystoneOS services own domain behavior. Graphical interfaces and CLI commands are clients of those services.

D-Bus is the default IPC mechanism. Private Unix sockets are reserved for cases where D-Bus is a poor fit, such as high-volume transfer or streaming paths.

## Service Rules

Each service should have:

- Narrow responsibility
- Minimal privileges
- Versioned API
- Documented D-Bus interface
- Structured logs
- Unit tests
- Integration tests where applicable
- CLI test client
- Explicit error behavior
- Migration rules for persistent data
- Authorization checks
- Graceful restart behavior where applicable

Do not create a daemon when a short-lived process is sufficient.

## Phase 1 Service Set

The first real service work should focus on:

- `waystone-projectd`
- `waystone-publishd`
- `waystone-identityd`
- `waystone-hostd`
- `waystone-audiod`

Other services remain planned but deferred:

- `waystone-serviced`
- `waystone-libraryd`
- `waystone-updated`
- `waystone-sessiond`
- `waystone-hardwared`

## waystone-projectd

Owns:

- Project discovery
- Project creation
- Manifest parsing
- Manifest validation
- Project inspection
- Project migration
- Project repair
- Project export metadata
- Project file inventory
- Project history access

Does not own:

- SSH credentials
- Remote transfer
- Audio encoding
- Host trust state
- GUI project planning or milestones

Initial operations:

```text
CreateProject
ListProjects
OpenProject
InspectProject
ValidateProject
ArchiveProject
CloneProject
ExportProject
RepairProject
MigrateProject
```

Persistent data:

- `.wayproject/project.toml`
- project history records
- project validation cache where needed

Primary clients:

- `project` CLI
- Waystone Workspace project views
- Future Helm integration
- Publishing service
- Audio service

## waystone-publishd

Owns:

- Publication preparation
- Validation orchestration
- Publication comparison
- Dry-run transfer plans
- Rsync/SCP/SFTP/Titan/Git/removable publication
- Remote verification
- Publication history recording
- Rollback metadata

Does not own:

- Raw SSH key storage
- Project manifest ownership
- Terminal sessions
- Audio export
- Service hosting

Initial operations:

```text
PreviewPublication
ValidatePublication
BuildPlannedHistory
BuildCompletedHistory
SaveCompletedHistory
ListCompletedHistory
ReadCompletedHistory
```

Future operations:

```text
PreparePublication
ComparePublication
PreviewTransfer
Publish
VerifyRemote
RecordPublication
ListPublicationHistory
```

Current implementation is limited to non-mutating preview, publication
readiness validation, optional caller-supplied local remote-state comparison,
planned-history generation, completed-history record construction from explicit
result fields, local planned-preview saves under project `history/previews/`,
read-only saved-preview listing and detail loading from that project-local
directory, local completed-history saves under project `history/completed/`,
and read-only completed-record listing and detail loading from that
project-local directory. `BuildCompletedHistory`,
`SaveCompletedHistory`, `ListCompletedHistory`, and `ReadCompletedHistory` are
exposed through `waystone-publishd`. `SaveCompletedHistory` writes only local
project history under `history/completed/`.
It does not probe remote hosts, perform transfers, execute deletions, unlock
credentials, or verify remote results.

Safety requirements:

- Dry-run first for destructive actions
- Explicit confirmation for remote deletes
- Credentials redacted from logs and errors
- Remote verification recorded separately from transfer success

Primary clients:

- `publish` CLI
- Publish workspace
- Future Browser preview integration
- Future Comm host inspection handoff

## waystone-identityd

Owns:

- User identities
- SSH key references
- Gemini client certificates
- Server certificates
- Feed identity metadata
- Signing key references
- Credential export/import workflows
- Expiration warnings

Does not own:

- Project metadata
- SSH transport
- Remote host trust decisions beyond identity linkage
- Raw publication transfer

Initial operations:

```text
CreateIdentity
ListIdentities
InspectIdentity
ExportIdentity
ImportIdentity
LockIdentity
RemoveIdentity
CreateCertificate
InspectCertificate
RenewCertificate
ExportCertificate
TrustCertificate
RevokeCertificate
```

Security requirements:

- Store credentials in encrypted workspace locations.
- Never include private credentials in ordinary project exports.
- Warn before credential export.
- Redact secrets from logs and errors.

Primary clients:

- `identity` CLI
- `certificate` CLI
- Publishing service
- Host service
- Future Browser certificate selection

## waystone-hostd

Owns:

- Host records
- Addresses
- Protocols
- SSH host keys
- Trust state
- Service associations
- Publication destination references
- Host notes
- Diagnostics metadata

Does not own:

- Project manifests
- SSH private keys
- Terminal emulator behavior
- Firewall policy for local services

Initial operations:

```text
CreateHost
ListHosts
InspectHost
UpdateHost
RemoveHost
InspectHostKey
TrustHostKey
DiagnoseHost
ListHostServices
```

Security requirements:

- SSH host-key changes must be visible.
- Trust decisions must be explicit.
- Diagnostics must not log credentials.

Primary clients:

- `host` CLI
- `connect` CLI
- Publishing service
- Future Comm integration

## waystone-audiod

Owns:

- Audio device enumeration
- Source selection
- Level state
- Recording
- Interrupted-recording recovery
- Master file metadata
- Basic trim/split/join/fade/normalize operations
- Opus export
- Audio project attachment
- Feed enclosure metadata handoff

Does not own:

- Project manifest as source of truth
- Feed generation by itself
- Full DAW session state
- Arbitrary plugin hosting

Initial operations:

```text
ListAudioDevices
SelectInput
SelectOutput
CheckLevels
StartRecording
PauseRecording
StopRecording
RecoverRecording
ExportAudio
AttachAudioToProject
InspectAudioMetadata
```

Safety requirements:

- Microphone state must be explicit.
- Recording interruption must be recoverable where practical.
- Original masters must be preserved.
- Media metadata is untrusted input.

Primary clients:

- `record` CLI
- `listen` CLI
- Create workspace
- Audio views
- Project service
- Publishing service

## Cross-Service Rules

- `waystone-projectd` owns project structure.
- `waystone-publishd` owns publication preview now, and later publication execution and history.
- `waystone-identityd` owns identities and credentials.
- `waystone-hostd` owns host trust and destination metadata.
- `waystone-audiod` owns audio capture and export.

No service should reach directly into another service's persistent data without an API boundary.

## Initial D-Bus Naming

Provisional names:

```text
org.waystone.Project1
org.waystone.Publish1
org.waystone.Identity1
org.waystone.Host1
org.waystone.Audio1
```

Provisional object roots:

```text
/org/waystone/Project
/org/waystone/Publish
/org/waystone/Identity
/org/waystone/Host
/org/waystone/Audio
```

The `1` suffix marks the interface generation, not the product version.
