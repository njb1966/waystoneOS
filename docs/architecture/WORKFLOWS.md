# WaystoneOS Workflow Maps

Status: Draft for Phase 0
Date: 2026-07-17

Workflows are the main test for WaystoneOS architecture. A component is valuable when it advances a real WaystoneOS workflow without turning the system into a generic Linux desktop.

## First Vertical Slice

This is the first meaningful complete workflow:

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

During the OS-first phase, Helm, Browser, and Comm are integration targets and add-on applications. The OS work should define project, publication, host, identity, audio, and CLI/service contracts before depending on those applications.

## New Small-Web User

1. Boot WaystoneOS from USB.
2. Select Temporary Session or Create Encrypted Workspace.
3. Connect to Wireless or Wired network.
4. Open Explore.
5. Read the local small-web introduction.
6. Browse Gemini and Gopher resources.
7. Create a capsule project.
8. Write `index.gmi`.
9. Preview the capsule.
10. Publish to a host or export it.

Success criterion:

A technically competent newcomer can publish a basic capsule without manually configuring a server, generating certificates through raw commands, or learning systemd.

## Writer

1. Open a project.
2. Draft Gemtext.
3. Organize entries and support files.
4. Preview formatting and links.
5. Spell-check.
6. Generate or update a feed.
7. Publish.
8. Archive the publication state.

Success criterion:

WaystoneOS can serve as a complete text-first writing and publishing workstation.

## Audio Publisher

1. Select a microphone.
2. Check input levels.
3. Record a WAV or FLAC master.
4. Trim the recording.
5. Normalize loudness.
6. Enter metadata.
7. Export to Opus.
8. Attach the file to a Gemtext entry.
9. Add it to an RSS or Atom feed.
10. Publish text, media, and feed together.
11. Verify remote playback.

Success criterion:

A user can produce an audio gemlog or podcast-style entry without external production software.

## Pubnix User

1. Select a saved host.
2. Authenticate using an SSH identity.
3. Open a remote terminal.
4. Edit or synchronize files.
5. Check permissions and quotas.
6. Transfer content.
7. Verify the published result.
8. Disconnect cleanly.

Success criterion:

WaystoneOS is credible as a daily-use pubnix workstation.

## Pubnix Administrator

1. Open Operate.
2. Select one or more hosts.
3. Inspect SSH host keys.
4. Monitor known services.
5. Open remote terminals.
6. Inspect certificates.
7. Perform DNS and TLS diagnostics.
8. Transfer configuration files.
9. Save session notes.
10. Review service activity.

Success criterion:

WaystoneOS functions as a portable operations console for independent systems.

## Educator

1. Boot a teacher node.
2. Start a private classroom network.
3. Have student devices discover the teacher node.
4. Create temporary student identities.
5. Assign local capsule names.
6. Let students create and publish local sites.
7. Browse student sites without internet access.
8. Export student work before shutdown.

Success criterion:

A classroom exercise can be completed entirely over a disconnected local network.

## Older-Hardware User

1. Boot on supported older x86-64 hardware.
2. Obtain working graphics and networking.
3. Browse small-web protocols responsively.
4. Write and publish.
5. Play audio.
6. Suspend and resume.

Success criterion:

The system remains useful on a defined low-resource reference machine.

## Workflow Ownership

| Workflow Step | Owning OS Component | Notes |
| --- | --- | --- |
| Create/open project | Project service | GUI and CLI use the same service |
| Write Gemtext | Workspace app plus project service | Project format remains inspectable |
| Preview content | Project and protocol services | Local preview must not require network |
| Validate links/feed | Project/publishing service | Include malformed input tests |
| Record audio | Audio service | PipeWire details hidden in normal UI |
| Export Opus | Audio service | Preserve WAV/FLAC masters |
| Attach to project | Project and audio services | Metadata is versioned |
| Configure destination | Host, identity, publishing services | Credentials excluded from project export |
| Publish over SSH | Publishing service | Dry-run, compare, verify, record history |
| Inspect remote host | Host service and terminal add-on | Comm integration later |
| Browse result | Protocol dispatcher and browser add-on | Browser integration later |

## Deferral Rule

If a proposed feature does not materially advance one of these workflows, it should be deferred unless it removes a concrete blocker in security, accessibility, hardware support, recovery, or release engineering.

