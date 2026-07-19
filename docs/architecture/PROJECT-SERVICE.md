# WaystoneOS Project Service

Status: Draft for Phase 0
Date: 2026-07-17

`waystone-projectd` owns Waystone project structure and project metadata. It is the first service to define because publishing, audio attachment, host integration, future Helm integration, and CLI behavior all depend on stable project state.

## Responsibility

`waystone-projectd` owns:

- Project discovery
- Project creation
- Manifest parsing
- Manifest validation
- Project inspection
- Project file inventory
- Project migration
- Project repair
- Project archive/export metadata
- Project history access

`waystone-projectd` does not own:

- SSH private keys
- Remote transfer execution
- Audio recording or encoding
- Feed transfer
- Host trust decisions
- Helm goals, milestones, or task state
- Browser rendering
- Terminal sessions

## Persistent Data

The service reads and writes:

```text
*.wayproject/project.toml
*.wayproject/history/
*.wayproject/cache/
```

`project.toml` is the source of truth for project metadata.

History records are append-oriented project records, not audit logs for the entire OS.

Cache data must be disposable.

## Trust Rules

- Project paths are user-controlled input.
- Project manifests are untrusted until parsed and validated.
- Paths in project metadata must remain inside the project root unless a field is explicitly defined as a remote path.
- The service must reject `..` traversal in portable project paths.
- The service must reject absolute local paths in portable project metadata.
- The service must handle symlinks carefully and must not follow them across trust boundaries by default.
- Credentials are never stored in project files.

## Initial D-Bus Interface Sketch

Provisional bus name:

```text
org.waystone.Project1
```

Provisional object root:

```text
/org/waystone/Project
```

The `1` suffix marks the interface generation.

Current adapter status:

- `CreateProject` implemented on `waystone-projectd`
- `ListProjects` implemented on `waystone-projectd`
- `InspectProject` implemented on `waystone-projectd`
- `ValidateProject` implemented on `waystone-projectd`
- D-Bus service file and systemd user unit are present under `services/projectd/`
- D-Bus autostart is smoke-tested with a generated temporary service file
- Project migration, repair, archive, and export over D-Bus are not implemented

## Methods

### CreateProject

Creates a new `.wayproject` directory.

Inputs:

```json
{
  "parent": "/workspace/Projects/Capsules",
  "id": "long-century",
  "name": "Long Century",
  "type": "capsule",
  "content_index": "index.gmi"
}
```

Output:

```json
{
  "project_path": "/workspace/Projects/Capsules/long-century.wayproject",
  "schema": 1
}
```

Rules:

- Refuse to overwrite an existing directory.
- Refuse project IDs with path separators.
- Create the minimum required structure for text projects.
- Create audio/feed scaffold defaults for `audio-series` and `mixed-publication` projects.

### ListProjects

Lists known projects within a workspace or search root.

Inputs:

```json
{
  "root": "/workspace/Projects"
}
```

Output:

```json
{
  "projects": [
    {
      "id": "long-century",
      "name": "Long Century",
      "type": "capsule",
      "path": "/workspace/Projects/Capsules/long-century.wayproject",
      "schema": 1
    }
  ]
}
```

Rules:

- Do not recursively scan unbounded filesystem trees.
- Ignore directories that are not `.wayproject` unless explicitly requested.

### OpenProject

Loads and validates the manifest enough to establish project identity.

Inputs:

```json
{
  "path": "/workspace/Projects/Capsules/long-century.wayproject"
}
```

Output:

```json
{
  "id": "long-century",
  "name": "Long Century",
  "type": "capsule",
  "schema": 1,
  "valid": true
}
```

### InspectProject

Returns structured project metadata.

Inputs:

```json
{
  "path": "/workspace/Projects/Capsules/long-century.wayproject"
}
```

Output:

```json
{
  "schema": 1,
  "id": "long-century",
  "name": "Long Century",
  "type": "capsule",
  "content_root": "content",
  "content_index": "index.gmi",
  "feed_enabled": true,
  "publish_targets": ["production"],
  "warnings": []
}
```

### ValidateProject

Validates project structure and manifest rules.

Inputs:

```json
{
  "path": "/workspace/Projects/Capsules/long-century.wayproject",
  "level": "standard"
}
```

Output:

```json
{
  "valid": false,
  "errors": [
    {
      "code": "missing_content_index",
      "message": "content/index.gmi is missing",
      "path": "content/index.gmi"
    }
  ],
  "warnings": []
}
```

Validation levels:

- `basic`: manifest shape, required fields, and local paths
- `standard`: basic plus required files and target metadata
- `publication`: standard plus feed, URI, MIME, destination, and publication checks

### ArchiveProject

Prepares a safe project archive.

Rules:

- Exclude credentials by default.
- Include publication history unless explicitly excluded.
- Include enough metadata to restore the project later.

### ExportProject

Exports public project content or a full project copy.

Export modes:

- `public-content`
- `project-copy`
- `diagnostic`

Credential export is not part of project export.

### RepairProject

Attempts a safe repair.

Rules:

- Requires dry-run support.
- Must report every planned rewrite.
- Must preserve unknown fields where practical.

### MigrateProject

Migrates a project schema version.

Rules:

- Never silently migrate.
- Support dry-run.
- Record migration history.
- Preserve unknown fields where practical.

## Error Codes

Initial project service error codes:

| Code | Meaning |
| --- | --- |
| `project_not_found` | Project path does not exist |
| `manifest_missing` | `project.toml` is missing |
| `manifest_unreadable` | Manifest could not be read |
| `manifest_parse_failed` | Manifest is not valid TOML or cannot be decoded |
| `unsupported_schema` | Schema version is not supported |
| `required_field_missing` | Required manifest field is missing |
| `unsupported_project_type` | Project type is not supported |
| `invalid_project_path` | Path is absolute, traverses upward, or is otherwise unsafe |
| `missing_content_root` | Content root does not exist |
| `missing_content_index` | Content index does not exist |
| `duplicate_publish_target` | Publish target names are not unique |
| `unsupported_publish_method` | Publish target method is not supported |
| `credential_leak_risk` | Project appears to include private credential material |
| `migration_required` | Project schema requires migration before operation |
| `operation_cancelled` | User cancelled operation |

## CLI Mapping

| CLI | Service Method |
| --- | --- |
| `project create` | `CreateProject` |
| `project list` | `ListProjects` |
| `project open` | `OpenProject` |
| `project inspect` | `InspectProject` |
| `project validate` | `ValidateProject` |
| `project archive` | `ArchiveProject` |
| `project export` | `ExportProject` |
| `project repair` | `RepairProject` |
| `project migrate` | `MigrateProject` |

## Version 0.1 Cut

For version 0.1, implement first:

- Project creation
- Bounded project listing
- Manifest loading
- Basic validation
- Standard validation for required local files
- Human and JSON output through the CLI
- Example valid and invalid project fixtures

Current implementation status is tracked in [../development/IMPLEMENTATION-STATUS.md](../development/IMPLEMENTATION-STATUS.md).

D-Bus activation remains a documented placeholder until the direct daemon and test-session-bus path are stable.
