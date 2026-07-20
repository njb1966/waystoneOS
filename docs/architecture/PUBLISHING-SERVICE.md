# WaystoneOS Publishing Service

Status: Current non-mutating contract
Date: 2026-07-18

`waystone-publishd` owns publication preparation, validation orchestration, transfer planning, transfer execution, remote verification, and publication history.

Publishing must be boring, inspectable, and recoverable. A successful transfer is not the same as a verified publication.

## Responsibility

`waystone-publishd` owns:

- Publication preparation
- Validation orchestration
- Publication comparison
- Dry-run transfer plans
- Transfer execution
- Remote verification
- Publication history records
- Rollback metadata

It does not own:

- Project manifest structure
- SSH private key storage
- Host trust state
- Audio export
- Terminal sessions
- Local service firewall policy

## Supported Methods

Initial priority:

1. `rsync`
2. `scp`
3. `sftp`
4. `titan`
5. `git`
6. `local-service`
7. `removable`

Version 0.1 should model `rsync`, `scp`, `sftp`, and `removable`. It may implement only dry-run behavior initially.

## Pipeline

```text
Prepare
   |
   v
Validate
   |
   v
Compare
   |
   v
Preview
   |
   v
Transfer
   |
   v
Verify remotely
   |
   v
Record publication
```

Every stage has its own result. Later stages must not hide earlier warnings.

## Dry-Run Model

Dry-run output must include:

- Project ID
- Target name
- Destination method
- Destination URL or host reference
- Identity reference
- Host resolution state where host metadata is provided
- Identity resolution state where identity metadata is provided
- Files to upload
- Files to update
- Files to delete
- Files to skip
- Validation status
- Verification plan
- Required confirmations

Example:

```json
{
  "status": "ok",
  "schema": 1,
  "data": {
    "project": "long-century",
    "target": "production",
    "method": "rsync",
    "identity": "nick-pub",
    "destination": "gemini://example.org",
    "changes": {
      "upload": ["content/index.gmi"],
      "update": ["feeds/feed.xml"],
      "delete": [],
      "skip": []
    },
    "validation": {
      "valid": true,
      "warnings": []
    },
    "verification": {
      "remote_uri": "gemini://example.org/index.gmi",
      "checks": ["fetch", "hash", "mime"]
    },
    "confirmations": []
  }
}
```

Dry-run must not claim transfer or verification success.

## Destructive Deletion

Remote deletion is allowed only after preview.

Rules:

- Deletions must be listed explicitly.
- The user must confirm deletion unless target policy forbids deletion entirely.
- `--yes` may not silently confirm deletion unless the command already names the target and deletion policy explicitly.
- Publication history must record deleted files.
- Rollback metadata should include enough information to restore or explain deleted files where practical.

## Publication History Record

History records should be stored under the project `history/` directory as inspectable TOML or JSON.

The current implementation can generate a planned history record from a dry-run. Planned records are previews only; they must not be written as completed publication history because no transfer or verification has occurred.

Current planned TOML shape:

```toml
[publication]
schema = 1
date = "2026-07-17T12:00:00Z"
project_id = "long-century"
target = "production"
method = "rsync"
identity = "nick-pub"
destination = "gemini://example.org"
transfer_result = "planned"
verification_result = "not-run"

[[files]]
path = "content/index.gmi"
action = "planned-upload"

[rollback]
available = false
notes = "Dry-run only; no remote state changed"
```

## Remote Verification

Verification should be independent of transfer completion.

Initial verification checks:

- Fetch expected URI where protocol support exists.
- Confirm MIME type where available.
- Confirm content hash where practical.
- Confirm feed availability when feed was updated.
- Confirm audio enclosure availability when audio was published.

## D-Bus Interface

Current bus name:

```text
org.waystone.Publish1
```

Current object root:

```text
/org/waystone/Publish
```

Implemented methods:

```text
PreviewPublication
BuildPlannedHistory
```

Future methods may include `PreparePublication`, `ValidatePublication`,
`ComparePublication`, `PreviewTransfer`, `Publish`, `VerifyRemote`,
`RecordPublication`, and `ListPublicationHistory`.

## Error Codes

| Code | Meaning |
| --- | --- |
| `project_invalid` | Project validation failed |
| `target_not_found` | Publish target does not exist |
| `host_not_trusted` | Host trust state blocks publication |
| `identity_missing` | Required identity is unavailable |
| `credential_locked` | Required credential is locked |
| `transfer_failed` | Transfer command failed |
| `remote_verification_failed` | Remote verification failed |
| `delete_confirmation_required` | Remote deletion needs explicit confirmation |
| `quota_exceeded` | Destination appears to exceed quota |
| `operation_cancelled` | User cancelled the operation |

## Version 0.1 Cut

For version 0.1, define and test dry-run behavior before real remote mutation.

The current implementation supports a local, non-mutating dry-run plan that
lists publishable project files for a selected target and reports feed
readiness for configured feeds. Feed readiness includes the configured feed
path, whether the feed XML file exists, how many valid prepared feed-entry
sidecars target that feed, how many feed-entry sidecars are invalid, and
per-invalid-sidecar diagnostic paths with validation issue text. It can also
resolve host and identity metadata when local metadata roots are provided,
generate planned publication history records without writing them as completed
history, save planned preview records under project `history/previews/`, list
saved planned preview records, and read selected saved preview TOML through the
`publish` CLI. Preview saving is a local project write only. Saved preview
reads are constrained to the selected project's `history/previews/` directory.
These preview operations are available through the `publish` CLI; non-mutating
preview and planned-history generation are also available through the
`waystone-publishd` D-Bus adapter. It does not generate feeds automatically,
compare remote state, perform transfer, delete files, access credentials, probe
SSH host keys, or verify a remote result.

Current implementation status is tracked in [../development/IMPLEMENTATION-STATUS.md](../development/IMPLEMENTATION-STATUS.md).

Actual transfer remains deferred until project validation, host identity, and credential boundaries are stable.
