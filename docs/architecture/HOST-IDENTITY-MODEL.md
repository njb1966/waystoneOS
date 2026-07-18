# WaystoneOS Host and Identity Model

Status: Draft for Phase 0
Date: 2026-07-17

Hosts and identities are separate objects. Projects may reference them, but projects do not own credentials or host trust state.

## Goals

- Keep credentials out of project files.
- Make SSH host-key trust explicit.
- Support publishing over SSH without exposing raw key management in normal workflows.
- Support Gemini client certificates and publication identities.
- Keep identity export risky and explicit.
- Provide future Waystone Comm and Browser integration points without depending on those apps.

## Host Record

Host records describe known local or remote machines.

Initial fields:

```toml
[host]
id = "offgridholdout"
display_name = "Offgrid Holdout"
address = "offgridholdout.org"
notes = "Pubnix and publication host"

[[services]]
type = "ssh"
port = 22
trust = "trusted"

[[services]]
type = "gemini"
port = 1965
trust = "observed"
```

## Host Trust States

| State | Meaning |
| --- | --- |
| `unknown` | Host has not been checked |
| `observed` | Host was seen but not explicitly trusted |
| `trusted` | User accepted this host identity |
| `changed` | Known host key or certificate changed |
| `blocked` | User or policy blocks this host |

Host-key changes must be visible before publication or remote sessions continue.

## Identity Record

Identity records describe user identities and references to credential material.

Initial fields:

```toml
[identity]
id = "nick-pub"
display_name = "Nick"
author_name = "Nick"

[[ssh_keys]]
id = "pubnix-main"
public_key = "ssh-ed25519 AAAA..."
private_key_ref = "workspace-secret:ssh/pubnix-main"

[[certificates]]
id = "gemini-client"
type = "gemini-client"
fingerprint = "sha256:..."
private_key_ref = "workspace-secret:certs/gemini-client"
expires = "2027-07-17"
```

Private keys are referenced, not stored directly in project manifests or host records.

## Credential Storage Boundary

Credentials belong in encrypted workspace storage.

Project files may reference:

- Identity IDs
- Host IDs
- Certificate fingerprints
- Public keys

Project files must not contain:

- Private keys
- Passwords
- Tokens
- Credential-bearing URIs
- Decrypted certificate private keys

## SSH Host-Key Behavior

On first contact:

1. Record observed key.
2. Show fingerprint.
3. Require user trust before publication.

On key change:

1. Mark host `changed`.
2. Block publication by default.
3. Show old and new fingerprints.
4. Require explicit user action to trust the new key.

## D-Bus Interface Sketch

Host service:

```text
org.waystone.Host1
/org/waystone/Host
```

Identity service:

```text
org.waystone.Identity1
/org/waystone/Identity
```

Initial host methods:

```text
CreateHost
ListHosts
InspectHost
UpdateHost
RemoveHost
InspectHostKey
TrustHostKey
DiagnoseHost
ListHostServices
```

Initial identity methods:

```text
CreateIdentity
ListIdentities
InspectIdentity
ExportIdentity
ImportIdentity
LockIdentity
RemoveIdentity
CreateCertificate
InspectCertificate
RenewCertificate
ExportCertificate
TrustCertificate
RevokeCertificate
```

## Version 0.1 Cut

Version 0.1 should define host and identity records well enough for publication dry-runs.

The current implementation provides metadata-only host and identity record loading, listing, inspection, and validation. It does not store secrets, probe SSH host keys, unlock credentials, or contact remote hosts.

Current implementation status is tracked in [../development/IMPLEMENTATION-STATUS.md](../development/IMPLEMENTATION-STATUS.md).

Real secret storage and host-key probing remain deferred until the publishing implementation is ready to mutate remote systems.

