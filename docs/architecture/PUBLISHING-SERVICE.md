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
- Remote comparison state when caller-supplied local state is provided
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
    "comparison": {
      "configured": true,
      "source": "/tmp/remote-state.txt",
      "remote_paths": 2
    },
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

## Current Remote-State Comparison

The current implementation can compare local publishable files against a
caller-supplied local remote-state manifest. The manifest is a plain text file:
one remote-relative path per line, with blank lines and `#` comments ignored.
Absolute paths and parent-directory traversal are rejected.

This comparison is local-only. It does not contact a remote host, unlock
credentials, probe host keys, transfer files, delete files, or verify remote
content.

Current classification:

- Local file absent from the supplied state: `upload`
- Local file present in the supplied state: `skip`
- State path absent locally and target delete policy allows delete planning:
  `delete`
- State path absent locally and target delete policy is `forbid`: `skip`
- `update` is reserved until remote-state metadata includes hashes or other
  content comparison data

Current helper commands:

- `publish --export-remote-state --project PATH --target NAME [--output PATH]`
  emits the selected project's local publishable path set in the same plain
  text manifest format.
- `publish --inspect-remote-state --remote-state PATH` validates and lists an
  existing local manifest using the same parser as dry-run comparison.

`--export-remote-state --output PATH` uses create-new file semantics and refuses
to overwrite an existing file. These helpers are local inspection/export tools;
they do not contact or mutate remote systems.

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

The current implementation can also build and save a separate completed
history record from a dry-run plus explicit caller-supplied result fields.
Completed records are written under project `history/completed/` by the
`publish` CLI. This is a local write-result contract only; it does not execute
remote transfer or perform remote verification.

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

Current completed TOML uses the same inspectable shape, but records explicit
results:

```toml
[publication]
schema = 1
date = "2026-07-20T12:00:00Z"
project_id = "long-century"
target = "production"
method = "rsync"
identity = "nick-pub"
destination = "gemini://example.org"
transfer_result = "completed"
verification_result = "passed"

[[files]]
path = "content/index.gmi"
action = "planned-upload"

[rollback]
available = false
notes = "No rollback snapshot recorded"
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
ValidatePublication
TransferIntent
BuildPlannedHistory
BuildCompletedHistory
SaveCompletedHistory
ListCompletedHistory
ReadCompletedHistory
```

Future methods may include `PreparePublication`, `ComparePublication`,
`PreviewTransfer`, `Publish`, `VerifyRemote`, `RecordPublication`, and
`ListPublicationHistory`.

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
produce a non-mutating publication readiness report with `valid`, `blocked`,
`errors`, and `warnings`, generate planned publication history records without
writing them as completed history, save planned preview records under project
`history/previews/`, list saved planned preview records, read selected saved
preview TOML, build completed history result records from explicit
caller-supplied result fields, save those completed records under project
`history/completed/`, list saved completed records, and read selected completed
TOML through the `publish` CLI. It can export the local publishable path set as
a remote-state manifest, inspect existing local remote-state manifests, and
compare a dry-run against a caller-supplied local manifest without contacting a
remote. It can also build a non-mutating transfer intent report that recomputes
validation and dry-run state, reports whether execution would be ready, lists
blocking issues and required confirmations, shows change buckets and
host/identity resolution summaries, and identifies the future completed-history
directory without executing transfer. The publish planning crate, publish
service crate, and `publish` CLI can also build a non-mutating removable
executor preparation plan with a bounded local destination root and per-file
source/destination operation records. That preparation contract blocks
unsupported methods, existing transfer-intent blockers, and delete operations;
it does not copy files, delete files, create directories, write completed
history, call D-Bus, or contact a remote. Preview and completed-record saving
are local project writes only.
Saved preview reads are constrained to the selected project's
`history/previews/` directory, and completed-record reads are constrained to
the selected project's `history/completed/` directory. These preview,
validation, local comparison, helper, preparation, and local history operations are
available through the `publish` CLI; preview, validation, transfer-intent
reporting, planned-history generation, completed-history result-record
generation, and completed-history save/list/read are also available through the
`waystone-publishd` D-Bus adapter. D-Bus completed-history saving is a local
project write only. The transfer-intent D-Bus method is read-only; it is not a
mutating executor. It does not generate feeds automatically, probe remote
state, perform transfer, execute deletions, access credentials, probe SSH host
keys, or verify a remote result.

Current implementation status is tracked in [../development/IMPLEMENTATION-STATUS.md](../development/IMPLEMENTATION-STATUS.md).

Actual transfer remains deferred until project validation, host identity, and
credential boundaries are stable. The current transfer-readiness audit is
tracked in
[../development/PUBLISH-TRANSFER-READINESS-AUDIT.md](../development/PUBLISH-TRANSFER-READINESS-AUDIT.md).
