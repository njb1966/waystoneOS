# WaystoneOS Project Charter

Status: Draft for Phase 0
Date: 2026-07-17

## Purpose

WaystoneOS is a portable small-web workstation for exploring, writing, listening, recording, publishing, hosting, teaching, and administering independent network services.

WaystoneOS uses the Linux kernel and mature Linux hardware infrastructure internally, but it must not present itself as a conventional Linux distribution. The user-facing system is organized around independent places, publications, identities, hosts, recordings, connections, and workspaces.

## Product Boundary

The first public form of WaystoneOS is a persistent x86-64 live USB for ordinary laptops and desktops.

The first release path is:

1. Build a Debian-hosted development preview.
2. Validate the Waystone Workspace, terminology, and core workflows.
3. Build shared services and CLI contracts.
4. Complete text and audio publication vertical slices.
5. Move to a bootable image after the workflows are real.
6. Move to Yocto for the production image after the prototype proves the product.

Internal storage installation, custom compositor work, signed A/B updates, classroom mode, ARM64 support, and broad NVIDIA support are later milestones unless the project owner explicitly changes scope.

## Primary Users

WaystoneOS is designed for:

- Small-web newcomers
- Writers and text-first publishers
- Audio publishers and independent radio users
- Pubnix users
- Pubnix and small-server administrators
- Educators teaching open protocols
- Digital minimalists
- Retrocomputing users who need modern TLS and hardware support
- Owners of older x86-64 laptops and thin clients
- Users who want a portable independent-network workstation

## Native Concepts

Normal WaystoneOS interfaces should use these concepts:

- Project
- Publication
- Host
- Service
- Identity
- Certificate
- Feed
- Recording
- Library
- Connection
- Destination
- Workspace

Normal interfaces should not require users to understand Linux implementation details such as package managers, systemd units, network interface names, raw firewall rules, bootloader branding, kernel logs, or Unix filesystem internals.

Advanced implementation details belong in Workshop Mode, technical documentation, diagnostics, development tools, and license notices.

## Non-Goals

WaystoneOS is not:

- A general-purpose desktop operating system
- A lightweight Linux distribution
- A Debian or Ubuntu remix
- A conventional office workstation
- A gaming operating system
- A commercial-web browsing platform
- A desktop customization distribution
- A complete professional DAW
- A server distribution with a desktop added
- A platform for arbitrary package installation in normal mode
- A nostalgic recreation of Solaris, CDE, or OpenWindows

## Version 0.1 Scope

Version 0.1 is a development preview running as a dedicated Wayland session on Debian.

It should include:

- Waystone graphical workspace prototype
- Four workspaces: Explore, Create, Publish, Operate
- Activity navigation
- Project manager
- Basic Gemtext editor
- Terminal integration
- Waystone Browser launch and integration points
- Waystone Comm launch and integration points
- Waystone Helm project integration points
- Audio playback
- Basic audio recording
- Mock service management
- CLI framework

Version 0.1 validates terminology, workflow shape, GUI and CLI parity, and the authoring and publishing model. It is not expected to be a bootable WaystoneOS image.

## First Complete Workflow

The first meaningful vertical slice is:

```text
Open project in Waystone Helm
        |
        v
Write Gemtext
        |
        v
Record audio
        |
        v
Export Opus
        |
        v
Attach recording
        |
        v
Generate feed
        |
        v
Publish over SSH
        |
        v
Inspect host in Waystone Comm
        |
        v
Open result in Waystone Browser
```

This workflow is the early project yardstick. Infrastructure that does not advance it should be deferred unless it removes a concrete blocker.

## Product Family Boundary

WaystoneOS is part of the Waystone product family:

- Waystone Browser: multiprotocol exploration
- Waystone Helm: project awareness, direction, milestones, and next actions
- Waystone Comm: terminal, pubnix, host, and independent communications
- WaystoneOS: the operating environment and shared services

Browser, Helm, and Comm must remain usable as standalone applications on ordinary Linux systems. Inside WaystoneOS they receive deeper integration through shared services.

WaystoneOS must not become an application bundle. Its value comes from shared system concepts, services, storage, identity, publishing, audio, hosting, recovery, and security boundaries.

## Service Boundary

Graphical applications and CLI commands must call shared Waystone services rather than duplicating domain behavior.

Initial intended services:

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

Services should be narrow, privilege-aware, versioned, documented, logged structurally, and testable.

## Security Baseline

WaystoneOS assumes threats including stolen USB devices, hostile public Wi-Fi, malicious downloads, untrusted servers, exposed local services, credential leakage, malicious feeds, vulnerable media decoders, tampered updates, and Workshop Mode changes.

Baseline controls include:

- Encrypted persistent workspaces
- Temporary sessions with no retained keys
- Signed system images
- Read-only or image-based base system
- Explicit public-service exposure
- Least privilege service design
- Host-key and certificate inspection
- No removable-media autorun
- No execution of downloaded content by default
- Secret redaction in logs and errors
- Recovery paths for boot, workspace, display, network, and user-data export

Security must not be weakened merely to simplify a demo.

## Accessibility Baseline

Version 1 must include:

- Full keyboard use
- Visible focus
- Interface and font scaling
- High contrast
- Reduced motion
- Screen-reader compatibility
- Semantic labels
- Color-independent status indicators
- Keyboard media controls
- Playback speed controls
- Mono-audio option
- Transcript attachment support

Prototype UI work should consider 1366x768 displays, HiDPI, keyboard operation, and focus behavior from the beginning.

## Repository Direction

Use a WaystoneOS monorepo during early development unless repository boundaries become clearly justified.

Expected early structure:

```text
docs/
compositor/
workspace/
apps/
services/
cli/
integration/
protocols/
project-format/
packaging/
yocto/
images/
installer/
recovery/
tests/
tools/
examples/
```

Do not create this full structure before it is needed. Add directories when they have concrete content.

## Licensing Policy

WaystoneOS uses the MIT License as the project-wide default license.

The root `LICENSE` file applies to source code and project documentation unless a file or directory states a different license.

Third-party assets, fixtures, fonts, media samples, firmware references, package metadata, and vendor code must preserve their own licenses and attribution requirements. Future asset-specific exceptions should be documented in an ADR.

## Definition of Phase 0 Complete

Phase 0 is complete when the repository contains:

- Project charter
- Stable non-goals
- Shared terminology
- User workflow maps
- Version 0.1 scope
- Initial security model
- Architecture decision register
- Initial accepted ADRs
- Licensing decision
- Repository structure sufficient for Phase 1
