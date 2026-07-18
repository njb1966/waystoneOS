# ADR-0001: Product Identity and First Release Format

Status: Accepted
Date: 2026-07-17

## Context

WaystoneOS needs a clear product identity before implementation starts. The project can easily drift into being a lightweight Linux distribution, a desktop theme, or a bundled image containing Waystone applications.

The project plan defines WaystoneOS as a portable small-web workstation and identifies a persistent x86-64 live USB as the first public form.

## Decision

WaystoneOS is a complete portable small-web operating environment, not a Linux distribution with software removed.

The first public release format is a persistent x86-64 live USB for ordinary laptops and desktops.

Normal user-facing language should use WaystoneOS concepts such as Project, Publication, Host, Service, Identity, Certificate, Feed, Recording, Library, Connection, Destination, and Workspace.

WaystoneOS may accurately document that it uses the Linux kernel and mature Linux hardware infrastructure, but public product identity should not present it as Waystone Linux, a Debian remix, or a lightweight Linux distribution.

## Consequences

- Hardware detection must run at every boot.
- Encrypted persistence is a core feature.
- Temporary nonpersistent sessions must exist.
- Installation to internal storage is deferred.
- The live system is the product, not a demo edition.
- Normal interfaces must avoid exposing ordinary Linux administration details.
- Documentation must distinguish implementation substrate from product identity.

## Alternatives Considered

- Conventional Linux distribution: rejected because it makes package management, generic desktop expectations, and distribution identity central.
- Application bundle on a custom image: rejected because it lacks shared Waystone system concepts and services.
- Install-only operating system: rejected for the first release because portability is central to the mission.

## Follow-Up

- Document first-boot flows for temporary and encrypted persistent sessions.
- Define reference x86-64 hardware targets.
- Record branding and terminology rules in user documentation.

