# WaystoneOS Security Model

Status: Draft for Phase 0
Date: 2026-07-17

Security is part of the WaystoneOS workflow model. It should reduce risk without making ordinary small-web use needlessly technical.

## Security Goals

- Protect user data on a lost or stolen USB device.
- Keep the base system recoverable and difficult to mutate accidentally.
- Make public service exposure explicit.
- Keep credentials separate from ordinary project exports.
- Verify remote hosts, certificates, and updates.
- Treat network, protocol, feed, and media input as hostile.
- Provide useful diagnostics without leaking secrets.
- Preserve user agency through inspectable data and explicit recovery options.

## Non-Goals

- WaystoneOS is not an anonymity-focused operating system in the first release.
- WaystoneOS is not a hardened security research distribution.
- WaystoneOS does not promise safety for arbitrary downloaded executables.
- WaystoneOS does not expose raw firewall or service management as the normal user workflow.

Tor, I2P, hardware-backed keys, and stronger compartmentalization may be future work, but they are not required for the first vertical slices.

## Threats

Expected threats include:

- Stolen USB devices
- Malicious downloads
- Hostile servers
- Compromised remote hosts
- Hostile public Wi-Fi
- Exposed local services
- Unsafe compatibility applications
- Tampered updates
- Credential leakage
- Vulnerable media decoders
- Malformed feeds
- Malformed protocol responses
- Workshop Mode changes
- Removable-media failure
- Privilege escalation
- Path traversal
- Unsafe symlink handling

## Core Controls

### Storage Protection

- Persistent workspaces are encrypted.
- Temporary sessions retain no keys or history after shutdown.
- Credentials are excluded from ordinary project exports.
- Workspace export must distinguish public content from private identity material.
- Interrupted writes must be recoverable where practical.

### System Integrity

- The base system is read-only or image-based in normal mode.
- Updates use signed metadata and image verification.
- A failed update must roll back to a working system.
- Workshop Mode must be visibly distinct from normal mode.

### Network Exposure

- Local services are disabled until configured.
- Public exposure requires explicit confirmation.
- Access profiles should use user terms: This computer only, Local network, VPN, Public internet, Custom.
- Firewall rules are generated from service intent.
- SSH password login is disabled by default for Waystone-managed services.

### Identity and Trust

- SSH host-key changes must be visible.
- Certificates must expose fingerprint, expiration, and trust state.
- Exporting credentials requires warnings.
- Expiration warnings are part of normal identity and certificate management.

### Content Handling

- Downloaded content does not execute automatically.
- Removable media has no autorun behavior.
- Protocol parsers treat external input as hostile.
- Feed, certificate, URI, and media metadata parsing require malformed-input tests.
- Media decoding should be isolated where practical.

### Logging and Diagnostics

- Logs are structured and understandable.
- Secrets are never logged.
- Credentials are redacted from errors.
- Security-relevant events should be visible enough for administrators without burdening normal users.

## Security Boundaries

| Boundary | Requirement |
| --- | --- |
| Normal mode to Workshop Mode | Explicit transition and visible state |
| GUI/CLI to services | Narrow versioned APIs with authorization checks |
| Project data to credentials | Credentials excluded unless explicitly exported |
| Local-only service to public service | Explicit exposure confirmation |
| Downloaded content to execution | No automatic execution |
| Base system to workspace | Read-only/image base, encrypted user data |
| Current update to next update | Signed verification and rollback |

## Phase 0 Security Decisions Still Needed

- Workspace encryption details and recovery-key model
- Initial authorization model for D-Bus services
- Secret storage design
- Update mechanism evaluation criteria
- Service sandboxing baseline
- Security event taxonomy
- Workshop Mode transition model

These should become ADRs before implementation depends on them.

