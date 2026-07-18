# WaystoneOS Architecture Documents

This directory holds Phase 0 architecture documents for the operating system itself.

Start here:

- [CHARTER.md](CHARTER.md): product boundary, Phase 0 definition, and first workflow
- [TERMINOLOGY.md](TERMINOLOGY.md): user-visible WaystoneOS vocabulary
- [WORKFLOWS.md](WORKFLOWS.md): workflow maps used to validate architecture
- [SECURITY-MODEL.md](SECURITY-MODEL.md): initial threats, controls, and boundaries
- [PROJECT-FORMAT.md](PROJECT-FORMAT.md): versioned `.wayproject` storage model
- [PROJECT-SERVICE.md](PROJECT-SERVICE.md): project service contract
- [SERVICE-BOUNDARIES.md](SERVICE-BOUNDARIES.md): first-slice service ownership
- [SERVICE-CONTRACTS.md](SERVICE-CONTRACTS.md): current Rust service contracts and future D-Bus names
- [DBUS-ADAPTER-PLAN.md](DBUS-ADAPTER-PLAN.md): first D-Bus adapter sequencing and verification gates
- [PUBLISHING-SERVICE.md](PUBLISHING-SERVICE.md): publishing dry-run, transfer, verification, and history model
- [HOST-IDENTITY-MODEL.md](HOST-IDENTITY-MODEL.md): host trust and credential boundary model
- [AUDIO-METADATA.md](AUDIO-METADATA.md): recording metadata sidecar model
- [WORKSPACE-UI-PLAN.md](WORKSPACE-UI-PLAN.md): first Waystone Workspace UI planning boundary
- [CLI-STANDARDS.md](CLI-STANDARDS.md): native command behavior and output rules
- [VERSION-0.1-SCOPE.md](VERSION-0.1-SCOPE.md): development-preview scope
- [REPOSITORY-BOUNDARY.md](REPOSITORY-BOUNDARY.md): repo-local work boundary and add-on app policy

Architecture decisions are recorded separately in [../decisions](../decisions).

## Current Phase

The project is in early OS implementation after Phase 0 charter and architecture.

Current work should keep architecture documents synchronized with implemented service crates, CLIs, fixtures, and placeholder daemons. New implementation scaffolding should still be tied to an approved boundary or decision.
