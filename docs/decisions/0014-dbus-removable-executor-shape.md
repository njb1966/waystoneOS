# ADR-0014: D-Bus Removable Executor Shape

Status: Accepted
Date: 2026-07-21

Implementation note: `waystone-publishd` implemented this D-Bus method on
2026-07-21 with private-session-bus smoke coverage.

## Context

WaystoneOS now has a bounded local/removable file-copy executor through the
publish service crate and `publish --execute-removable`. It recomputes
preparation immediately before execution, requires explicit transfer
confirmation, refuses upload overwrites, copies through destination-directory
temporary files, records completed, failed, and partial copy-time outcomes, and
writes completed-history TOML from executor results.

`waystone-publishd` currently exposes preview, validation, transfer-intent, and
history operations over D-Bus, but it intentionally does not expose removable
execution. Before adding any mutating publish D-Bus method, the request,
response, error, and UI boundaries need to be explicit.

## Decision

`org.waystone.Publish1.ExecuteRemovable` may expose only the existing
removable executor behavior. It must use a schema-versioned JSON string request
and response matching the existing adapter pattern.

Request shape:

```json
{
  "schema": 1,
  "project_path": "/path/to/project.wayproject",
  "target": "export",
  "remote_state_path": "/path/to/removable-state.txt",
  "date": "2026-07-21T00:00:00Z",
  "confirm_transfer": true
}
```

Rules:

- `confirm_transfer` is required and must be exactly `true`.
- There is no generic `yes`, `force`, or implicit confirmation field.
- `remote_state_path` is optional and remains caller-supplied local state, not a
  remote probe.
- Host roots, identity roots, credential references, and delete confirmation
  are not accepted by this method.
- The method must delegate to `PublishService::execute_removable`.
- The service crate remains the owner of executor behavior.
- The adapter must not execute deletes, call SSH tools, unlock credentials,
  probe host keys, or verify remote results.

Successful transport response shape:

```json
{
  "schema": 1,
  "ok": true,
  "data": {
    "project": "audio-capsule",
    "target": "export",
    "method": "removable",
    "destination_root": "/path/to/project.wayproject/publish/export",
    "transfer_result": "partial",
    "verification_result": "not-run",
    "files": [
      {
        "project_path": "content/index.gmi",
        "source_path": "/path/to/project.wayproject/content/index.gmi",
        "destination_path": "/path/to/project.wayproject/publish/export/content/index.gmi",
        "action": "upload",
        "result": "failed",
        "bytes": null,
        "error": "destination directory could not be created"
      }
    ],
    "history": {
      "completed_path": "/path/to/project.wayproject/history/completed/20260721T000000Z-export.toml",
      "record": {}
    }
  }
}
```

`ok: true` means the D-Bus method executed the service operation and, if
copy-time execution began, returned the executor-produced result and
completed-history location. It does not mean the transfer completed. Callers
must inspect `transfer_result`.

`transfer_result` values for this method are currently:

- `completed`
- `failed`
- `partial`

`verification_result` is currently always `not-run`.

Error response shape:

```json
{
  "schema": 1,
  "ok": false,
  "error": {
    "code": "confirmation_required",
    "message": "removable execution requires confirm_transfer"
  }
}
```

Expected error codes:

- `invalid_request`
- `confirmation_required`
- `removable_execution_plan_failed`
- `removable_execution_blocked`
- `removable_execution_preflight_failed`
- `completed_history_write_failed`

Preflight, planning, confirmation, and completed-history write failures return
`ok: false`. Copy-time failures that are represented in executor results return
`ok: true` with `transfer_result = "failed"` or `"partial"` if history was
written from that result.

## Consequences

- D-Bus mutation has a reviewed, narrow shape before implementation.
- `publishd` implementation can keep smoke coverage against a private test
  session bus without changing the service crate contract.
- Qt mutating publish controls remain deferred until the D-Bus method is
  implemented, smoke-tested, and explicitly approved for UI exposure.
- SSH-family execution remains outside this contract.

## Alternatives Considered

Expose a generic `Publish` method:

- Rejected because it would blur removable execution, SSH execution, deletion,
  credential unlock, and verification before those boundaries are ready.

Expose `ExecuteRemovable` with a generic `yes` or `force` flag:

- Rejected because publication mutation should require explicit,
  operation-specific confirmation.

Return `ok: false` for partial executor outcomes:

- Rejected because partial and failed copy-time outcomes are successful IPC
  deliveries of executor-produced results. Treating them as transport errors
  would hide the completed-history record and per-file result details from
  callers.

Implement the D-Bus method immediately in the same slice:

- Deferred so the contract can be reviewed and documented before adding a
  mutating IPC surface.

## Follow-Up

- Keep `waystone-publishd` implementation aligned with this contract as the
  method evolves.
- Keep private-session-bus smoke coverage before any Qt mutating publish UI.
- Keep real SSH-family executors, delete execution, credential unlock, host-key
  probing, and remote verification deferred.
