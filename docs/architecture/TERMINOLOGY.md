# WaystoneOS Terminology

Status: Draft for Phase 0
Date: 2026-07-17

WaystoneOS should use a stable vocabulary that describes what users are doing, not the Linux mechanisms underneath.

## Naming Rules

Use:

- WaystoneOS
- WaystoneOS 0.x
- WaystoneOS Portable Small-Web Workstation
- Waystone Workspace
- Workshop Mode

Avoid:

- Waystone Linux
- Waystone OS
- WayStoneOS
- Debian remix
- lightweight Linux distribution
- Linux desktop environment
- desktop shell, when referring to the graphical workspace

Technical documentation may describe Linux, systemd, Wayland, wlroots, PipeWire, NetworkManager, nftables, OpenSSH, and similar implementation components. Normal user interfaces should not require those terms.

## Core Objects

### Workspace

The user's portable WaystoneOS environment.

A workspace contains projects, writing, audio, library items, connections, identities, certificates, service definitions, destinations, and settings.

Normal use should expose the workspace, not Unix home-directory internals.

### Project

A structured publication or work unit.

Examples:

- Capsule
- Gemlog
- Gopherhole
- Spartan site
- Audio series
- Feed
- Pubnix home
- Documentation archive
- Classroom assignment
- Mixed publication

Projects must remain understandable outside WaystoneOS. Project state should use versioned, inspectable formats.

### Publication

A recorded publishing event or publishable output from a project.

A publication includes destination, changed files, deleted files, hashes, validation results, remote verification, identity, and rollback data.

### Host

A local or remote machine known to WaystoneOS.

Hosts may have addresses, protocols, SSH host keys, identities, services, publication destinations, notes, and trust state.

### Service

A local or remote network service managed or observed by WaystoneOS.

Examples:

- Gemini
- Gopher
- Spartan
- Nex
- Finger
- SSH
- Feed host
- Audio stream
- Local preview
- Classroom directory

### Identity

A person, author, or operational identity used by WaystoneOS.

An identity may include display name, SSH keys, Gemini client certificates, server certificates, feed metadata, signing keys, associated hosts, and expiration dates.

### Certificate

A TLS, Gemini client, server, or trust object that WaystoneOS can create, inspect, renew, export, trust, or revoke.

Certificate interfaces should expose purpose, subject, issuer, fingerprint, expiration, trust state, and associated projects or hosts.

### Feed

A structured publication stream.

Supported feed concepts include RSS, Atom, Gemsub, Twtxt, and audio enclosure feeds.

### Recording

An audio capture associated with a workspace or project.

Recordings may have master files, publication copies, metadata, markers, normalization state, and feed attachment state.

### Library

The user's saved material.

Library items include bookmarks, saved pages, feed entries, audio, images, archives, local mirrors, and downloads.

### Connection

An active or saved relationship to a host, service, identity, protocol endpoint, network, or destination.

Examples include SSH sessions, SFTP targets, Gemini capsule bookmarks, VPN profiles, pubnix accounts, and classroom connections.

### Destination

A configured target for publication or export.

Examples include rsync over SSH, SCP, SFTP, Titan, Git, local service publication, and removable-media export.

## Activities

Primary WaystoneOS activities:

- Explore
- Write
- Listen
- Record
- Publish
- Host
- Connect
- Learn
- Operate

Default workspaces:

- Explore
- Create
- Publish
- Operate

Activities describe user intent. Workspaces organize the interface.

## User-Visible Storage

Normal users should see:

```text
Workspace
|-- Projects
|-- Writing
|-- Audio
|-- Library
|-- Connections
|-- Services
`-- Storage
```

Normal users should not be sent into `/etc`, `/var`, `/usr`, `/sys`, or similar system structures.

Workshop Mode may expose the implementation filesystem for development, diagnostics, and advanced administration.

## Command Vocabulary

Native command names:

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

Commands should support human-readable output, JSON output where useful, meaningful exit codes, `--help`, noninteractive operation, dry-run behavior, safe defaults, and explicit destructive confirmation.

