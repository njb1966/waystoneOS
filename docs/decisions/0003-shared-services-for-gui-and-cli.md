# ADR-0003: Shared Services for Graphical and CLI Behavior

Status: Accepted
Date: 2026-07-17

## Context

WaystoneOS must provide both a graphical workspace and a capable command-line environment. If graphical applications and CLI tools implement domain behavior separately, project state, permissions, publishing behavior, and error handling will diverge.

The project plan identifies shared Waystone services as the layer that makes WaystoneOS more than an application bundle.

## Decision

Major graphical and CLI operations must call shared Waystone services.

D-Bus is the default IPC mechanism for domain services. Private Unix sockets may be used where D-Bus is demonstrably unsuitable, such as high-volume transfer or streaming paths.

Initial intended services include:

- waystone-projectd
- waystone-publishd
- waystone-identityd
- waystone-hostd
- waystone-serviced
- waystone-audiod
- waystone-libraryd
- waystone-updated
- waystone-sessiond
- waystone-hardwared

Each service must have a narrow responsibility, versioned API, documented interface, structured logs, tests, explicit error behavior, and privilege-aware authorization.

## Consequences

- GUI and CLI parity becomes an architectural requirement.
- Public schemas and D-Bus interfaces must be versioned from the beginning.
- Service logic must not be hidden inside widgets or CLI command handlers.
- Tests must cover both GUI-to-service and CLI-to-service paths where applicable.
- Service boundaries need to be designed before large application features are built.

## Alternatives Considered

- Put logic directly in graphical applications: rejected because it breaks CLI parity and standalone testability.
- Put logic directly in CLI tools: rejected because it makes the GUI a wrapper around command execution.
- Use one giant daemon: rejected because it would create unclear ownership and excessive privilege.

## Follow-Up

- Define initial project service API.
- Define CLI output standards, including JSON output and exit codes.
- Create a D-Bus interface documentation convention.

