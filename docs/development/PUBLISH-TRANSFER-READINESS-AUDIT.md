# Publish Transfer Readiness Audit

Status: current after D-Bus removable executor shape ADR
Date: 2026-07-21

This audit records the boundary between the current local/removable publishing
model and any future command that would execute a real remote transfer.

The goal is to prevent WaystoneOS from adding `rsync`, `scp`, or `sftp`
execution before the safety, credential, history, and verification contracts
are explicit enough to keep publication boring, inspectable, and recoverable.

## Current Foundation

The repository can perform bounded local/removable file-copy execution, but it
is not yet ready to perform SSH-family remote transfer.

Implemented foundations:

- `.wayproject` validation requires SSH-family targets to name `host`,
  `identity`, and `remote_path`.
- Project paths reject absolute paths and parent-directory traversal in
  portable project metadata.
- Publish dry-runs are non-mutating and include upload, update, delete, and
  skip buckets.
- Caller-supplied local remote-state manifests can be inspected, exported, and
  used for comparison without contacting a remote.
- Configured removable destination roots can be scanned into the same local
  state manifest shape without contacting a remote.
- Deletion planning is visible and governed by target `delete_policy`.
- Host and identity metadata can be resolved from local metadata roots.
- Publication readiness validation reports `valid`, `blocked`, `errors`, and
  `warnings`.
- `publish --transfer-intent` and `org.waystone.Publish1.TransferIntent`
  report execution readiness, blocking issues, required confirmations, change
  buckets, host/identity resolution summaries, comparison metadata, and the
  future completed-history directory without executing transfer.
- `publish --prepare-removable-execution` reports a bounded, non-mutating
  removable executor preparation plan with a local destination root and
  per-file source/destination operation records.
- Removable execution preparation blocks unsupported methods, existing
  transfer-intent blockers, and delete operations.
- `publish --execute-removable` copies upload/update files into the configured
  removable destination root after `--confirm-transfer`, refuses upload
  overwrites, copies through destination-directory temporary files before
  renaming into place, refuses stale temporary-copy collisions, and writes
  completed history from executor results.
- Copy-time removable execution failures are reported as structured failed or
  partial executor results with per-file error text. Completed-history TOML is
  written from those executor results when execution reaches the copy phase.
- `publish --export-removable-state` emits the configured removable
  destination root's current file path set so existing removable media can be
  fed back into dry-run comparison as caller-supplied local state.
- Planned history previews and completed-history records are inspectable local
  records.
- `waystone-publishd` exposes preview, validation, read-only transfer-intent,
  planned-history, and completed-history record operations over D-Bus.
- ADR-0014 defines the future `ExecuteRemovable` D-Bus mutating request and
  response shape, but the method is not implemented yet.

Important current limits:

- No command probes remote SSH host keys.
- No command unlocks or resolves private credentials.
- No command runs `rsync`, `scp`, `sftp`, or remote shell commands.
- No command deletes local removable or remote files.
- No command independently verifies remote publication results.
- Remote-state comparison has paths only; it has no remote hashes, sizes,
  mtimes, MIME data, or feed/audio verification data.
- Completed-history records can still be manually built from supplied result
  fields, but removable execution now writes completed history from executor
  results.

## Required Gates Before Remote Mutation

Real transfer execution should remain blocked until these gates are satisfied.

| Gate | Requirement | Current State |
| --- | --- | --- |
| Command boundary | A separate transfer command must be defined instead of overloading dry-run behavior. | Non-mutating `publish --transfer-intent` and `publish --prepare-removable-execution` implemented |
| Method scope | First executor must support only one method or one local-safe method class. | Local executor is bounded to `removable` |
| Credential boundary | Identity records must resolve to an execution credential without exposing secret material in output, logs, history, JSON, or D-Bus errors. | Deferred |
| Host trust | SSH host trust must be checked against host metadata before transfer. Unknown or mismatched trust must block. | Metadata exists; live probing deferred |
| Remote path safety | Remote target path handling must reject empty, root, home, traversal-like, and shell-expanded destinations. | Partially modeled in manifest; executor checks absent |
| Dry-run freshness | The transfer command must require a fresh validation/preview basis or recompute validation immediately before execution. | Preparation recomputes transfer intent and dry-run state immediately |
| Deletion confirmation | Planned deletes must require explicit delete confirmation separate from ordinary `--yes`. | Delete execution remains blocked |
| Failure semantics | Partial transfer, cancellation, network failure, and permission failure must have stable result states. | Local removable copy-time failures now write failed/partial result history; cancellation and network failure remain deferred |
| History source | Completed history must be generated from executor results, not manually supplied success claims. | Implemented for removable execution; manual result helpers still exist |
| Verification boundary | Transfer success and remote verification must remain separate result stages. | Documented; verifier absent |
| D-Bus contract | Any mutating publish method must have a reviewed request/response shape before UI use. | ADR-0014 defines future `ExecuteRemovable`; mutating method implementation remains deferred |
| Test harness | Real execution must be covered by a local fake transport or temporary destination harness before live SSH targets are used. | Temporary-project removable copy and removable-state export harnesses exist |

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

## Removable Executor Preparation Contract

The first bounded executor-facing command is:

```text
publish --prepare-removable-execution --project PATH --target NAME \
  [--remote-state PATH] [--json]
```

This command remains non-mutating. It recomputes transfer intent and dry-run
state, verifies that the selected target uses `method = "removable"`, resolves
the project-relative target path as a local destination root, and reports
per-file source/destination operation records for upload, update, delete, and
skip buckets. It does not copy files, delete files, create directories, write
completed history, call D-Bus, or contact a remote.

Current blockers:

- Any non-`removable` target reports `unsupported_executor_method`.
- Any transfer-intent blocker remains a preparation blocker.
- Any planned delete reports `delete_execution_not_supported`.

## Removable File-Copy Execution

The first real executor is local and bounded:

```text
publish --execute-removable --project PATH --target NAME --date DATE \
  --confirm-transfer [--remote-state PATH] [--json]
```

Current behavior:

- Recomputes the removable preparation plan immediately before execution.
- Requires `--confirm-transfer`.
- Copies upload/update files into the configured removable destination root.
- Uses destination-directory temporary files and renames into place after copy
  completion.
- Refuses upload overwrites when the destination path already exists.
- Refuses stale temporary-copy path collisions before copying starts.
- Records skipped files as skipped executor results.
- Writes completed history under the selected project `history/completed/`
  from executor results.
- Writes failed or partial completed-history records when copy-time execution
  fails after the copy phase begins.
- Prints structured JSON with per-file failure detail and exits nonzero unless
  the transfer result is `completed`.
- Leaves verification as `not-run`.
- Does not execute deletes, call D-Bus, contact remotes, unlock credentials, or
  probe SSH host keys.

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

1. Implement `ExecuteRemovable` in `waystone-publishd` only by following
   ADR-0014 and adding private-session-bus smoke coverage.
2. Consider Qt read-only ergonomics for removable state export only if it helps
   the local 0.1 demonstrable flow.
3. Keep SSH-family executors behind the credential, host-trust, remote-path,
   delete-confirmation, executor-history, and verification gates.

Still defer:

- `rsync`, `scp`, and `sftp` execution
- credential unlock
- SSH host probing
- remote deletion
- remote verification
- Qt mutating publish controls
