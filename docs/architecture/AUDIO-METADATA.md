# WaystoneOS Audio Metadata

Status: Current local metadata contract
Date: 2026-07-19

Audio metadata describes recordings and publication copies without requiring audio decoding or device access.

## Scope

Current implementation supports:

- TOML sidecar loading
- TOML sidecar creation for attaching an existing master/publication copy
- Recording listing
- Recording inspection
- Metadata validation
- Project-relative master and published paths
- Feed enclosure metadata references

Current implementation does not support:

- PipeWire device enumeration
- Recording capture
- Playback
- Trimming
- Normalization
- Opus export
- Audio codec inspection
- Waveform generation
- Metadata replacement or merge editing

## Sidecar Shape

Audio sidecars live under:

```text
audio/metadata/
```

Example:

```toml
[recording]
id = "field-note"
title = "Field Note"
master = "audio/masters/field-note.flac"
published = "audio/published/field-note.opus"
duration_seconds = 318
channels = 1
sample_rate = 48000

[publication]
feed = "feeds/feed.xml"
entry_id = "tag:example.invalid,2026:field-note"
mime_type = "audio/ogg; codecs=opus"
```

Paths are project-relative and must not be absolute or traverse upward with `..`.

## CLI Mapping

Current commands:

```text
record attach PROJECT ID TITLE MASTER PUBLISHED FEED ENTRY_ID MIME_TYPE
record list ROOT
record inspect PATH
record validate PATH
listen library ROOT
```

`record attach` creates one metadata sidecar under the selected project's
configured `[audio].metadata` root. It references an existing project-relative
master file, an existing project-relative publication copy, and a feed enclosure
handoff path. It does not copy files, record audio, export Opus, generate a feed,
or overwrite an existing sidecar.

The `record` command owns recording metadata creation and inspection. The
`listen` command can list playable recording metadata, but it does not play
audio yet.
