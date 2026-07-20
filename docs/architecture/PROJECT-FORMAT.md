# WaystoneOS Project Format

Status: Draft for Phase 0
Date: 2026-07-17

The Waystone project format is the durable on-disk model for user-owned publication work.

Project data must remain understandable outside WaystoneOS. The format should use directories, plain text, TOML, XML, Gemtext, audio files, and other conventional formats rather than opaque binary databases.

## Goals

- Represent small-web publication projects clearly.
- Support text and audio publishing.
- Keep user data portable between machines.
- Allow GUI and CLI operations to share the same source of truth.
- Support validation, publication history, migration, repair, and export.
- Keep credentials out of ordinary project exports.
- Let future Waystone Helm, Browser, and Comm integrations consume project state without owning it.

## Directory Layout

Canonical form:

```text
example.wayproject/
|-- project.toml
|-- content/
|   |-- index.gmi
|   |-- about.gmi
|   `-- gemlog/
|-- audio/
|   |-- masters/
|   |-- published/
|   `-- metadata/
|-- assets/
|-- feeds/
|   |-- feed.xml
|   |-- entries/
|   `-- gemsub.gmi
|-- templates/
|-- publish/
|   |-- staging.toml
|   `-- production.toml
|-- cache/
`-- history/
```

Only `project.toml` and `content/` are required for a minimal text project.
`audio-series` and `mixed-publication` projects are created with the audio and
feed directories shown above so recording attachment has a stable local target.

Other optional directories should be created lazily when a workflow needs them.

## Manifest Versioning

Every project manifest must declare a schema version.

```toml
[waystone]
schema = 1
created_by = "WaystoneOS"
```

Schema changes that affect persistent data require migration rules and tests.

## Minimal Manifest

```toml
[waystone]
schema = 1

[project]
id = "my-capsule"
name = "My Capsule"
type = "capsule"
language = "en"
author = "Nick"

[content]
root = "content"
index = "index.gmi"
```

## Full Example

```toml
[waystone]
schema = 1
created_by = "WaystoneOS"

[project]
id = "long-century"
name = "Long Century"
type = "capsule"
language = "en"
author = "Nick"
description = "A small-web writing and audio project"

[content]
root = "content"
index = "index.gmi"

[audio]
masters = "audio/masters"
published = "audio/published"
metadata = "audio/metadata"
master_format = "flac"
publish_format = "opus"
publish_bitrate = 96000

[feed]
enabled = true
type = "atom"
path = "feeds/feed.xml"
title = "Long Century"

[[publish.targets]]
name = "production"
method = "rsync"
host = "offgridholdout"
identity = "nick-pub"
remote_path = "/srv/gemini/nick"
url = "gemini://example.org"
delete_policy = "confirm"

[[publish.targets]]
name = "export"
method = "removable"
path = "publish/export"
```

## Project Types

Initial project types:

- capsule
- gemlog
- gopherhole
- spartan-site
- audio-series
- feed
- pubnix-home
- documentation-archive
- classroom-assignment
- mixed-publication

Unknown project types should produce a validation warning or error depending on the operation. They should not be silently treated as ordinary folders.

`audio-series` and `mixed-publication` are audio-capable project types. New
projects of those types receive `[audio]` and `[feed]` manifest defaults plus
`audio/masters`, `audio/published`, `audio/metadata`, and `feeds/feed.xml`.
The initial feed file is a placeholder. Feed-entry sidecar preparation is
implemented under `feeds/entries/`, and `record generate-feed` can replace the
placeholder with a minimal Atom feed generated from validated sidecars. When an
existing Atom feed is present, `record generate-feed` replaces sidecar-managed
entries by ID and preserves unrelated existing entries.

## Required Fields

Required:

- `waystone.schema`
- `project.id`
- `project.name`
- `project.type`
- `content.root`
- `content.index`

Recommended:

- `project.language`
- `project.author`
- `project.description`
- `feed.enabled`
- `publish.targets`

## Path Rules

- Paths in `project.toml` are relative to the project root.
- Absolute paths are not allowed in portable project metadata.
- `..` path traversal is not allowed.
- Symlinks must be handled carefully and never followed across trust boundaries by default.
- Project export must not include private credentials unless explicitly requested through a credential export workflow.
- Cache files may be regenerated and must not be required for project integrity.

## Publishing Targets

Supported target methods, in priority order:

1. `rsync`
2. `scp`
3. `sftp`
4. `titan`
5. `git`
6. `local-service`
7. `removable`

Targets reference host and identity objects by name or stable ID. Secrets do not live in `project.toml`.

Destructive remote deletion requires `delete_policy = "confirm"` or stronger explicit confirmation at publish time.

For `rsync`, `scp`, and `sftp`, targets require:

- `host`
- `identity`
- `remote_path`

For `removable`, targets require:

- `path`

Supported `delete_policy` values:

- `confirm`
- `forbid`

## Audio Metadata

Audio metadata should be stored as inspectable TOML sidecars under `audio/metadata/`.

Example:

```toml
[recording]
id = "2026-07-17-field-note"
title = "Field Note"
master = "audio/masters/2026-07-17-field-note.flac"
published = "audio/published/2026-07-17-field-note.opus"
duration_seconds = 318
channels = 1
sample_rate = 48000

[publication]
feed = "feeds/feed.xml"
entry_id = "tag:example.org,2026:field-note"
mime_type = "audio/ogg; codecs=opus"
```

Prepared feed-entry sidecars may be stored under `feeds/entries/`:

```toml
[entry]
id = "tag:example.org,2026:field-note"
title = "Field Note"
updated = "2026-07-19T00:00:00Z"
summary = "Field note summary"
feed = "feeds/feed.xml"
recording = "2026-07-17-field-note"
recording_metadata = "audio/metadata/2026-07-17-field-note.toml"

[enclosure]
path = "audio/published/2026-07-17-field-note.opus"
mime_type = "audio/ogg; codecs=opus"
```

## History

Publication history should be written under `history/` using append-only, inspectable records.

History records should include:

- Date
- Project schema version
- Project version or content hash
- Destination
- Changed files
- Deleted files
- Hashes
- Transfer result
- Verification result
- Identity reference
- Rollback information

## Validation Rules

Initial validation should check:

- Manifest schema version
- Required fields
- Project type
- Relative paths
- Missing files
- Invalid path traversal
- Unsupported publish methods
- Duplicate publish target names
- Duplicate feed IDs
- Malformed RSS or Atom
- Gemtext link syntax
- Missing audio files
- MIME type consistency
- Oversized audio files where a target has limits
- Private credential leakage

## CLI Operations

Initial project commands:

```text
project create
project target add-removable
project list
project open
project inspect
project validate
project archive
project clone
project export
project repair
project migrate
```

All commands should support useful human-readable output. Commands that inspect state should support JSON output.

## Migration

Migration must be explicit.

Rules:

- Never silently rewrite a project across schema versions.
- Provide dry-run migration.
- Back up or snapshot files before migration where practical.
- Preserve unknown fields unless the migration explicitly owns them.
- Record migration history.
