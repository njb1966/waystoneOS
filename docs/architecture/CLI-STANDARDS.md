# WaystoneOS CLI Standards

Status: Draft for Phase 0
Date: 2026-07-17

WaystoneOS keeps a capable Unix command line, but native commands should expose Waystone concepts rather than raw implementation details.

## Command Set

Initial native commands:

```text
way
project
explore
write
listen
record
publish
host
connect
identity
certificate
service
network
storage
library
update
system
help
```

Not every command needs implementation in version 0.1. Commands should appear when there is a real service or workflow behind them.

## Output Modes

Commands should default to human-readable output.

Commands that inspect state should support:

```text
--json
```

JSON output must be stable enough for scripts within a major interface version.

Human output should be concise, readable, and use Waystone terms.

## Exit Codes

Common exit codes:

| Code | Meaning |
| --- | --- |
| 0 | Success |
| 1 | General failure |
| 2 | Usage error |
| 3 | Validation failed |
| 4 | Not found |
| 5 | Permission or authorization denied |
| 6 | Network failure |
| 7 | Remote verification failed |
| 8 | Operation cancelled |
| 9 | Conflict or stale state |
| 10 | Unsupported schema or migration required |

Commands may define additional domain-specific codes only when necessary and documented.

## Common Flags

Use where applicable:

```text
--help
--json
--dry-run
--quiet
--verbose
--project NAME_OR_PATH
--workspace PATH
--target NAME
--identity NAME
--yes
--no-color
```

`--yes` may confirm routine prompts, but must not bypass dangerous operations unless the operation and risk are explicit in the command.

## Dry Run

Destructive or remote-changing operations should support `--dry-run`.

Publishing dry runs must show:

- Destination
- Files to upload
- Files to change
- Files to delete
- Validation status
- Identity reference
- Verification plan

Dry-run output must not claim success for operations that were not performed.

## Confirmation

Require explicit confirmation for:

- Remote deletion
- Credential export
- Identity removal
- Certificate revocation
- Workspace migration
- Project repair that rewrites files
- Service exposure to Public internet
- Update channel changes

Confirmation prompts should name the object and consequence directly.

## Error Messages

Errors should include:

- What failed
- Which object was affected
- Whether data was changed
- A useful next action

Errors must not include:

- Private keys
- Passwords
- Tokens
- Full credential-bearing URIs
- Secret environment values

Example:

```text
publish: validation failed for project "long-century"
missing file: content/about.gmi
no files were transferred
```

## JSON Shape

JSON output should include:

```json
{
  "status": "ok",
  "schema": 1,
  "data": {}
}
```

Failures should include:

```json
{
  "status": "error",
  "schema": 1,
  "error": {
    "code": "validation_failed",
    "message": "Project validation failed",
    "details": []
  }
}
```

Use machine-readable error codes. Human messages may change; codes should be stable within an interface generation.

## Progress and Cancellation

Long-running operations should show progress in human mode.

Examples:

- Recording
- Audio export
- Publication comparison
- Transfer
- Remote verification
- Update download

Operations should handle cancellation cleanly where practical. Cancellation should leave the system in a known state and report whether partial output remains.

## Service Relationship

Native commands are clients of Waystone services.

Do not place domain behavior only in CLI implementations. If a CLI operation becomes a product behavior, the owning service must expose it through a versioned API.

## Initial 0.1 Commands

Version 0.1 should prioritize:

```text
project create
project target add-removable
project list
project inspect
project validate
publish --dry-run
publish --planned-history
publish --save-planned-history-preview
host list
host inspect
identity list
record list-devices
record start
record stop
```

Commands may use mocked service responses during early UI validation, but the command contract should reflect the eventual service boundary.

The current implemented project CLI subset is tracked in [../development/IMPLEMENTATION-STATUS.md](../development/IMPLEMENTATION-STATUS.md).
