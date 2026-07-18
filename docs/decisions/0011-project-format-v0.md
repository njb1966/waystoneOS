# ADR-0011: Project Format v0

Status: Accepted
Date: 2026-07-17

## Context

WaystoneOS needs a durable project model before services, CLI commands, publishing, audio attachment, and future app integrations can be designed coherently.

The project format must remain inspectable outside WaystoneOS and portable between machines.

## Decision

Waystone projects are directories ending in `.wayproject` with a versioned TOML manifest named `project.toml`.

The format uses conventional files and directories for content, audio, assets, feeds, templates, publishing targets, cache, and history.

The initial schema version is `waystone.schema = 1`.

Project metadata must not contain private credentials. Publishing targets may reference hosts, destinations, and identities, but secrets live in the encrypted workspace identity/host stores.

## Consequences

- GUI, CLI, and services can share one inspectable source of truth.
- Project data can be exported, archived, repaired, and inspected without WaystoneOS.
- Migration rules are required for future schema changes.
- Path traversal, symlink handling, missing files, feed validity, and credential leakage need tests.
- Helm, Browser, and Comm can later consume project state through service APIs rather than owning the format.

## Alternatives Considered

- Opaque project database: rejected because it reduces user ownership and repairability.
- Flat folder with no manifest: rejected because publication, audio, feed, and destination state need a stable schema.
- Store credentials in project files: rejected because project export must be safe by default.

## Follow-Up

- Implement manifest parsing and validation in the project service.
- Add project format fixtures.
- Define migration test cases before changing schema version.
- Define host and identity references used by publish targets.

