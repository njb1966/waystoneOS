# Publish Transfer Readiness Audit

Status: current after non-mutating transfer-intent contract
Date: 2026-07-20

This audit records the boundary between the current non-mutating publishing
model and any future command that would execute a real remote transfer.

The goal is to prevent WaystoneOS from adding `rsync`, `scp`, or `sftp`
execution before the safety, credential, history, and verification contracts
are explicit enough to keep publication boring, inspectable, and recoverable.

## Current Foundation

The repository is ready to describe transfer execution, but not yet ready to
perform it.

Implemented foundations:

- `.wayproject` validation requires SSH-family targets to name `host`,
  `identity`, and `remote_path`.
- Project paths reject absolute paths and parent-directory traversal in
  portable project metadata.
- Publish dry-runs are non-mutating and include upload, update, delete, and
  skip buckets.
- Caller-supplied local remote-state manifests can be inspected, exported, and
  used for comparison without contacting a remote.
- Deletion planning is visible and governed by target `delete_policy`.
- Host and identity metadata can be resolved from local metadata roots.
- Publication readiness validation reports `valid`, `blocked`, `errors`, and
  `warnings`.
- `publish --transfer-intent` reports execution readiness, blocking issues,
  required confirmations, change buckets, host/identity resolution summaries,
  comparison metadata, and the future completed-history directory without
  executing transfer.
- Planned history previews and completed-history records are inspectable local
  records.
- `waystone-publishd` exposes preview, validation, planned-history, and
  completed-history record operations over D-Bus.

Important current limits:

- No command probes remote SSH host keys.
- No command unlocks or resolves private credentials.
- No command runs `rsync`, `scp`, `sftp`, or remote shell commands.
- No command deletes remote files.
- No command independently verifies remote publication results.
- Remote-state comparison has paths only; it has no remote hashes, sizes,
  mtimes, MIME data, or feed/audio verification data.
- Completed-history records can store supplied result fields, but they are not
  produced by a Waystone transfer executor.

## Required Gates Before Remote Mutation

Real transfer execution should remain blocked until these gates are satisfied.

| Gate | Requirement | Current State |
| --- | --- | --- |
| Command boundary | A separate transfer command must be defined instead of overloading dry-run behavior. | Non-mutating `publish --transfer-intent` implemented |
| Method scope | First executor must support only one method or one local-safe method class. | Not defined |
| Credential boundary | Identity records must resolve to an execution credential without exposing secret material in output, logs, history, JSON, or D-Bus errors. | Deferred |
| Host trust | SSH host trust must be checked against host metadata before transfer. Unknown or mismatched trust must block. | Metadata exists; live probing deferred |
| Remote path safety | Remote target path handling must reject empty, root, home, traversal-like, and shell-expanded destinations. | Partially modeled in manifest; executor checks absent |
| Dry-run freshness | The transfer command must require a fresh validation/preview basis or recompute validation immediately before execution. | Not defined |
| Deletion confirmation | Planned deletes must require explicit delete confirmation separate from ordinary `--yes`. | Policy modeled; execution confirmation absent |
| Failure semantics | Partial transfer, cancellation, network failure, and permission failure must have stable result states. | Deferred |
| History source | Completed history must be generated from executor results, not manually supplied success claims. | Manual result fields exist |
| Verification boundary | Transfer success and remote verification must remain separate result stages. | Documented; verifier absent |
| D-Bus contract | Any mutating publish method must have a reviewed request/response shape before UI use. | Deferred |
| Test harness | Real execution must be covered by a local fake transport or temporary destination harness before live SSH targets are used. | Deferred |

## Transfer Command Boundary

Live SSH mutation remains deferred.

The first non-executing transfer intent contract now answers:

- Which dry-run plan is being executed?
- Which target and method are allowed?
- Which confirmations were supplied?
- Which delete policy applies?
- Which host and identity were resolved?
- Which credential reference would be needed?
- Which files would be uploaded, skipped, updated, or deleted?
- Where would executor-produced history be written after a real run?

Implemented command shape:

```text
publish --transfer-intent --project PATH --target NAME \
  [--hosts ROOT] [--identities ROOT] [--remote-state PATH] [--json]
```

This command remains non-mutating. It reports blocked validation state,
resolved host and identity summaries where available, required confirmations,
change buckets, comparison metadata, the future completed-history directory,
and a clear `execution_ready` boolean. It does not access credentials, contact
a remote, or write completed history.

## First Executor Recommendation

After the intent contract is reviewed in use, the first real executor should
be local and bounded before SSH transfer:

```text
publish --execute-removable --project PATH --target NAME --confirm-transfer
```

Rationale:

- The `removable` method already uses a local project-relative target path.
- It exercises real file copy, overwrite, conflict, delete-policy, and history
  behavior without network credentials or SSH host trust.
- It can be tested inside temporary directories.
- It gives the Qt Publish pane a real completed-history producer without
  claiming remote SSH support.

Only after removable execution is stable should `rsync` be considered. `scp`
and `sftp` should remain behind the same gates, because they need credential
and host-trust behavior just as much as `rsync`.

## Deletion Policy

Deletion remains the highest-risk publication action.

Executor rules should be:

- `delete_policy = "forbid"` means no delete action may execute.
- `delete_policy = "confirm"` allows delete planning but not delete execution
  unless a delete-specific confirmation is present.
- Ordinary transfer confirmation must not imply delete confirmation.
- Delete history must record every deleted path.
- Rollback notes must say whether recovery material exists.

Recommended future flags:

```text
--confirm-transfer
--confirm-delete
```

`--yes` should not be accepted as a substitute for `--confirm-delete` unless a
later architecture decision deliberately changes this rule.

## History and Verification

Completed publication history should eventually be written from executor
results with at least:

- transfer result: completed, failed, skipped, partial, or cancelled
- verification result: not-run, passed, failed, or partial
- per-file attempted action
- per-file outcome where available
- delete outcomes
- rollback availability and notes
- command method and target

Remote verification should remain a separate stage. A transfer executor may
finish successfully while verification fails or remains not run.

## Recommended Next Slice

Choose the next boundary deliberately before any remote mutation.

Recommended implementation order:

1. Add `waystone-publishd` D-Bus exposure for the read-only transfer-intent
   report if the UI or service boundary needs it before execution work.
2. Otherwise, define the bounded `removable` executor contract and test harness
   before implementing file-copy behavior.
3. Keep SSH-family executors behind the credential, host-trust, remote-path,
   delete-confirmation, executor-history, and verification gates.

Still defer:

- `rsync`, `scp`, and `sftp` execution
- credential unlock
- SSH host probing
- remote deletion
- remote verification
- Qt mutating publish controls
