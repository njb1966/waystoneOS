# ADR-0006: Audio Scope for Version 1

Status: Accepted
Date: 2026-07-17

## Context

Audio is a first-class small-web workflow for WaystoneOS. The system should support listening, recording, preparing publication copies, feed enclosures, and audio publishing.

There is a risk that audio work expands into a full professional DAW, which would consume the project and dilute the small-web workstation mission.

## Decision

Version 1 includes audio playback, streaming, voice and music recording, WAV and FLAC masters, Opus publication copies, basic trimming, splitting, joining, fades, loudness normalization, metadata editing, feed enclosures, project attachment, publishing integration, independent radio support, and basic device routing.

Version 1 does not include full multitrack production, virtual instruments, MIDI sequencing, arbitrary plugin hosting, large mastering workflows, or complete DAW replacement functionality.

Use PipeWire, WirePlumber, ALSA, and GStreamer or FFmpeg libraries as the implementation base.

## Consequences

- Audio is part of the primary vertical slice.
- Device enumeration, recording, export, attachment, feed generation, publishing, and remote playback verification all need tests.
- Standard audio views should be simple; advanced routing belongs in a Studio view.
- The system should prioritize reliable publication workflows over professional production breadth.

## Alternatives Considered

- Exclude audio from early releases: rejected because audio publishing is central to the product.
- Build a full DAW: rejected because it is outside scope and would delay core workflows.
- Require external production tools: rejected for the core v1 workflow because WaystoneOS should support complete basic audio publication.

## Follow-Up

- Define audio project metadata.
- Define recording recovery behavior.
- Define publication presets and feed enclosure validation.

