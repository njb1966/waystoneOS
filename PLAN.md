Below is the consolidated project plan, with **WaystoneOS** as the working product name and Waystone Browser, Helm, and Comm incorporated as first-party applications.

# WaystoneOS Project Plan

## 1. Executive summary

WaystoneOS will be a portable, purpose-built operating system for exploring, creating, publishing, hosting, teaching, and administering the small web.

It will use the Linux kernel and established Linux hardware infrastructure internally, but it will not present itself as a conventional Linux distribution. Users will not encounter GNOME, KDE, XFCE, LXQt, a standard application menu, an app store, a visible package manager, or a conventional Linux administration model during normal use.

WaystoneOS will organize the computer around small-web concepts:

* Projects
* Publications
* Hosts
* Services
* Identities
* Certificates
* Feeds
* Recordings
* Libraries
* Connections
* Workspaces

The primary distribution format will be a persistent x86-64 live USB designed for ordinary laptops and desktops. Installation to an internal drive will be added after the portable technical preview is proven reliable.

WaystoneOS will be a member of the existing Waystone application family:

* Waystone Browser: multiprotocol browsing and exploration
* Waystone Helm: project awareness, progress, and direction
* Waystone Comm: terminals, remote hosts, pubnix connections, and independent communications
* WaystoneOS: the native operating environment integrating these applications with publishing, identity, audio, hosting, and system services

The operating system will use a restrained workstation interface informed by CDE, OpenWindows, and Solaris design principles. It should retain the visual economy and lightweight impression of those environments without becoming a literal retro reproduction.

Audio will be a first-class function. WaystoneOS will support playback, recording, basic editing, metadata, feed generation, audio publishing, independent radio, and audio resources distributed through small-web protocols.

---

# 2. Product definition

## 2.1 Product statement

> WaystoneOS is a portable small-web workstation for exploring, writing, listening, recording, publishing, hosting, teaching, and administering independent network services.

## 2.2 Core purpose

WaystoneOS exists to solve the fragmented experience currently faced by small-web users.

Today, users must separately discover and configure:

* Gemini browsers
* Gopher clients
* Spartan and Nex clients
* Gemtext editors
* terminal applications
* SSH tools
* certificate utilities
* feed readers
* audio tools
* Gemini and Gopher servers
* publishing scripts
* firewall rules
* remote-host configurations
* local preview servers

WaystoneOS will combine these functions into a coherent operating environment.

## 2.3 Primary audiences

WaystoneOS is intended for:

* Small-web newcomers facing a scattered onboarding process
* Writers seeking a distraction-free publishing workstation
* Audio publishers and independent radio enthusiasts
* Pubnix users
* Pubnix and small-server administrators
* Retrocomputing users who still need modern TLS and hardware support
* Digital minimalists
* Educators teaching protocols and decentralized publishing
* Owners of older laptops and thin clients
* Users carrying a portable independent-network workstation
* Gemini, Gopher, Spartan, Nex, Finger, RSS, Atom, and Twtxt users
* People who want to browse and publish without centering the commercial web

## 2.4 Non-goals

WaystoneOS will not attempt to become:

* A general-purpose desktop operating system
* A minimal Debian or Ubuntu derivative
* A generic lightweight Linux distribution
* A conventional office workstation
* A gaming operating system
* A commercial-web browsing platform
* A desktop-customization distribution
* A complete professional digital audio workstation
* A server distribution with a desktop added
* A replacement for every Linux application
* A platform for arbitrary package installation in normal mode
* An intentionally nostalgic recreation of Solaris, CDE, or OpenWindows

---

# 3. Confirmed project decisions

## 3.1 Primary release format

The first public form of WaystoneOS will be:

> A portable persistent live USB for ordinary x86-64 laptops and desktops.

Consequences:

* Hardware must be detected at every boot.
* User storage must be portable between machines.
* User persistence must be encrypted.
* Display, audio, and network profiles must adapt to changing hardware.
* Temporary nonpersistent sessions must be supported.
* Installation to an internal drive is deferred until after the technical preview.
* The live system is the primary product, not a secondary demonstration edition.

## 3.2 Normal command-line environment

WaystoneOS will use a hybrid model.

Normal mode will include:

* Bash or another conventional interactive Unix shell
* Waystone-native commands
* SSH
* SCP
* SFTP
* Rsync
* Git
* Vim or another terminal editor
* Grep
* Find
* Sed
* Awk
* Tar
* OpenSSL utilities
* DNS utilities
* Ping
* Traceroute
* MTR
* Network inspection
* File and archive tools
* User-level process inspection

Normal mode will not include:

* Visible package management
* Arbitrary modification of the base system
* Direct editing of system service units
* Installation of unmanaged kernel modules
* Replacement of core system libraries
* Unrestricted mutation of the immutable operating-system image

Workshop Mode will provide development, debugging, low-level administration, and package-building tools.

## 3.3 Audio scope

Version 1 will include:

* Audio playback
* Streaming
* Voice and music recording
* WAV and FLAC masters
* Opus publication copies
* Basic trimming
* Splitting and joining
* Fade-in and fade-out
* Loudness normalization
* Metadata editing
* Feed enclosure generation
* Project attachment
* Publishing integration
* Small-web radio and audio-resource support
* Basic routing and device management

Version 1 will not attempt to provide:

* Full multitrack production
* Virtual instruments
* MIDI sequencing
* Professional plugin hosting
* Large-scale mastering workflows
* Complete DAW replacement functionality

## 3.4 Visual direction

WaystoneOS will use:

> A modern functional interface based on Solaris workstation principles while retaining the visual economy of CDE and OpenWindows.

It should avoid an excessively modern appearance.

Desired qualities:

* Compact controls
* Strong window boundaries
* Clear menus
* Minimal animation
* High information density
* Visible system state
* Strong keyboard navigation
* Workspaces
* Hosts and services as major objects
* Restrained typography
* Simple symbolic icons
* Lightweight visual impression
* Clear use of rectangular and inset controls
* Functional rather than artistic presentation

Avoid:

* Floating translucent panels
* Rounded mobile-style cards
* Oversized padding
* Animated docks
* Search-only navigation
* Hidden hover controls
* Decorative blur
* Excessive shadows
* Consumer-cloud aesthetics
* Flat interfaces with poor boundary visibility
* Literal pixel-for-pixel imitation of CDE or OpenWindows

## 3.5 Product name

The working and likely permanent name is:

> WaystoneOS

Recommended styling:

* WaystoneOS
* WaystoneOS 0.1
* WaystoneOS Portable Small-Web Workstation

Avoid:

* Waystone Linux
* WayStoneOS
* Waystone OS
* Waystone Distribution

Technical documentation may state that WaystoneOS is built on the Linux kernel. The public identity should not present it as a Linux distribution.

---

# 4. Waystone product family

## 4.1 Product structure

The Waystone family will consist of:

### WaystoneOS

The complete operating environment.

Responsibilities:

* Hardware
* Sessions
* Workspaces
* identities
* certificates
* projects
* publications
* audio
* services
* hosts
* updates
* persistence
* recovery
* system security

### Waystone Browser

The principal Explore application.

Responsibilities:

* Gemini
* Gopher
* Spartan
* HTTP and HTTPS in standalone versions
* Nex
* Finger
* local documents
* feed discovery
* certificate-aware browsing
* project preview
* small-web bookmarks
* saved-page integration
* audio handoff
* protocol inspection

### Waystone Helm

The project-awareness and direction application.

Responsibilities:

* Project goals
* milestones
* tasks
* status
* health
* momentum
* current priorities
* next actions
* project relationships
* project overview

### Waystone Comm

The terminal and remote-communications application.

Responsibilities:

* Local terminal sessions
* SSH
* Telnet
* RLogin
* Serial
* ANSI BBS systems
* Pubnix access
* Remote-host bookmarks
* Session logging
* Host identity display
* File-transfer integration
* Service consoles

## 4.2 Product independence

Waystone Browser, Helm, and Comm should remain independently installable on ordinary Linux systems.

WaystoneOS will provide deeper integration through shared services.

This creates two distribution models:

### Standalone

Applications run on conventional Linux systems.

### Native WaystoneOS

Applications use WaystoneOS services for:

* Projects
* identities
* certificates
* hosts
* publishing
* audio
* libraries
* permissions
* system notifications

## 4.3 WaystoneOS must not become an application bundle

WaystoneOS cannot simply be:

* Yocto
* a custom panel
* Waystone Browser
* Waystone Helm
* Waystone Comm

It must provide shared system concepts beneath those applications.

```text
                         WaystoneOS Services
                                  │
          ┌───────────────────────┼───────────────────────┐
          │                       │                       │
 Waystone Browser          Waystone Helm           Waystone Comm
 Explore content           Direct projects         Connect to hosts
```

Shared operating-system services will distinguish WaystoneOS from a bundled Linux image.

---

# 5. Design principles

## 5.1 Purpose before size

WaystoneOS is not defined by having the smallest possible image.

A complete 2–4 GB system that supports browsing, writing, audio, publishing, hosting, and administration is preferable to a smaller image that requires users to assemble missing tools.

## 5.2 No implementation leakage

Normal users should not need to know that WaystoneOS uses:

* Linux
* systemd
* Wayland
* wlroots
* PipeWire
* WirePlumber
* NetworkManager
* nftables
* OpenSSH
* Rsync
* Agate
* Molly Brown
* FFmpeg
* GStreamer

These names may appear in:

* Workshop Mode
* Diagnostic output
* Technical documentation
* License notices
* Development tools

The normal interface should instead use:

* Project
* Publish
* Host
* Service
* Identity
* Certificate
* Recording
* Feed
* Connection
* Workspace
* Library

## 5.3 Graphical and CLI parity

The graphical and CLI interfaces must call the same underlying services.

```text
Graphical Publish action
           │
           ▼
   Publication service
           ▲
           │
      publish command
```

Every major graphical operation should have a CLI equivalent.

## 5.4 Immutable operating system

The base OS should be read-only or image-based during normal use.

User data should be separated from the system image.

The system should support:

* Signed updates
* A/B system partitions
* automatic rollback
* encrypted user workspaces
* a read-only recovery environment

## 5.5 Modern hardware without a consumer desktop

WaystoneOS must retain:

* Modern graphics
* Wi-Fi
* Ethernet
* Bluetooth
* Audio
* USB audio
* Multiple monitors
* HiDPI
* NVMe
* USB storage
* Suspend
* Resume
* Battery management
* Touchpads
* Brightness keys
* Firmware updates
* USB tethering
* Captive portal authentication

It will exclude unrelated general-purpose desktop applications.

## 5.6 Offline usefulness

Without internet access, users must still be able to:

* Write
* Record
* Edit audio
* Browse saved material
* Run local services
* Teach protocols
* Create projects
* Prepare publication bundles
* Export to removable storage
* Inspect certificates
* Use classroom networking

## 5.7 Functional simplicity

The interface should favor:

* Visible controls
* predictable menus
* direct navigation
* keyboard access
* clear system status
* concise terminology
* stable workflows

It should not favor novelty or decorative presentation.

---

# 6. Intended user workflows

## 6.1 New small-web user

1. Boot from USB.
2. Select a temporary or encrypted persistent workspace.
3. Connect to Wi-Fi.
4. Open Waystone Browser.
5. Read the local small-web introduction.
6. Browse Gemini and Gopher resources.
7. Create a capsule project.
8. Write `index.gmi`.
9. Preview the capsule.
10. Publish to a host or export it.

Success criterion:

A technically competent newcomer can publish a basic capsule without manually configuring a server, generating certificates through raw commands, or learning systemd.

## 6.2 Writer

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

## 6.3 Audio publisher

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

Success criterion:

A user can produce an audio gemlog or podcast entry without external production software.

## 6.4 Pubnix user

1. Select a saved host in Waystone Comm.
2. Authenticate using an SSH identity.
3. Open a remote terminal.
4. Edit or synchronize files.
5. Check permissions and quotas.
6. Transfer content.
7. Verify the published result.
8. Disconnect cleanly.

Success criterion:

WaystoneOS is credible as a daily-use pubnix workstation.

## 6.5 Pubnix administrator

1. Open the Operate workspace.
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

## 6.6 Educator

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

## 6.7 Older-hardware user

1. Boot on supported older x86-64 hardware.
2. Obtain working graphics and networking.
3. Browse small-web protocols responsively.
4. Write and publish.
5. Play audio.
6. Suspend and resume.

Success criterion:

The system remains useful on a defined low-resource reference machine.

---

# 7. Release roadmap

## 7.1 Version 0.1: Development preview

Runs as a dedicated Wayland session on Debian.

Includes:

* Waystone graphical workspace prototype
* Four workspaces
* Activity navigation
* Project manager
* Basic Gemtext editor
* Terminal
* Waystone Browser integration
* Waystone Comm integration
* Basic Waystone Helm project integration
* Audio playback
* Basic audio recording
* Mock service management
* CLI framework

Purpose:

* Validate workflows
* Validate terminology
* Validate product-family integration
* Test GUI and CLI parity
* Test the authoring and publishing model

## 7.2 Version 0.2: Bootable technical preview

Includes:

* Bootable x86-64 image
* Custom graphical workspace
* Existing wlroots compositor
* Temporary live mode
* Persistent live mode
* Wi-Fi and Ethernet
* Audio playback and recording
* Gemini and Gopher browsing
* Project creation
* SSH publishing
* Local preview
* Workshop Mode
* Recovery console

Not yet included:

* Custom compositor
* Full graphical installer
* A/B updates
* Classroom mode
* ARM64
* Broad NVIDIA support

## 7.3 Version 0.5: Public alpha

Includes:

* Dedicated wlroots compositor
* Encrypted persistence
* Signed image updates
* Better display management
* Integrated PipeWire audio
* Audio editing
* Project validation
* Identity and certificate management
* Local Gemini and Gopher hosting
* Remote-host management
* Native CLI
* Hardware reporting

## 7.4 Version 1.0: Initial stable release

Includes:

* Portable x86-64 live USB
* Install-to-disk option
* Immutable A/B system images
* Graphical recovery
* Encrypted workspace
* Custom compositor
* Complete GUI and CLI workflows
* Gemini
* Gopher
* Spartan
* Nex
* Finger
* RSS
* Atom
* Gemsub
* Twtxt where practical
* Text and audio publishing
* Local hosting
* Remote administration
* Accessibility baseline
* Signed updates
* Reproducible builds
* Local documentation
* Hardware compatibility records

---

# 8. High-level architecture

```text
┌──────────────────────────────────────────────────────────────┐
│                      WAYSTONE WORKSPACE                      │
│                                                              │
│ Explore  Write  Listen  Record  Publish  Host  Connect       │
├──────────────────────────────────────────────────────────────┤
│                   FIRST-PARTY APPLICATIONS                   │
│                                                              │
│ Waystone Browser   Waystone Helm   Waystone Comm             │
│ Text Studio        Audio Studio    Publication Desk          │
│ Host Desk          Library         System Settings           │
├──────────────────────────────────────────────────────────────┤
│                    COMMAND ENVIRONMENT                       │
│                                                              │
│ project  explore  listen  record  publish  host              │
│ connect  identity  certificate  service  network  update     │
├──────────────────────────────────────────────────────────────┤
│                    WAYSTONE SERVICES                         │
│                                                              │
│ Projects   Publishing   Identity   Hosts   Audio   Library   │
│ Services   Updates      Hardware   Session   Configuration   │
├──────────────────────────────────────────────────────────────┤
│                COMPOSITOR AND SESSION LAYER                  │
│                                                              │
│ wlroots compositor   workspaces   locking   display policy   │
├──────────────────────────────────────────────────────────────┤
│                   HIDDEN SYSTEM SERVICES                     │
│                                                              │
│ PipeWire  networking  SSH  TLS  firewall  power  storage     │
├──────────────────────────────────────────────────────────────┤
│                     HARDWARE PLATFORM                        │
│                                                              │
│ Linux kernel  Mesa  DRM/KMS  libinput  ALSA  drivers         │
└──────────────────────────────────────────────────────────────┘
```

---

# 9. Kernel and hardware platform

## 9.1 Kernel

Use a mainstream Linux LTS kernel.

Required support:

* x86-64
* UEFI
* ACPI
* Intel CPUs
* AMD CPUs
* Intel graphics
* AMD graphics
* DRM/KMS
* Mesa
* NVMe
* SATA
* USB 2, 3, and 4
* USB storage
* USB audio
* Ethernet
* common Wi-Fi chipsets
* Bluetooth
* touchpads
* external displays
* battery sensors
* thermal sensors
* suspend
* hibernation where reliable
* LUKS
* device mapper
* namespaces
* Landlock
* seccomp
* nftables
* WireGuard
* virtual-machine drivers

The kernel configuration must be version-controlled and reproducible.

## 9.2 Graphics

Hidden graphics infrastructure:

* DRM/KMS
* Mesa
* Wayland
* libinput
* XWayland during compatibility transition

Users should not encounter these terms in normal settings.

## 9.3 Networking

Hidden infrastructure may include:

* NetworkManager or systemd-networkd
* iwd
* WPA supplicant where needed
* nftables
* WireGuard
* OpenSSH
* mDNS
* DNS resolution
* DHCP
* USB tethering
* Bluetooth networking

The visible interface should use plain terms such as:

* Wired
* Wireless
* Remote Access
* Local Services
* VPN
* Connection Status

## 9.4 Power

Retain:

* Suspend
* Resume
* Lid actions
* Battery state
* Screen dimming
* Power profiles
* Thermal monitoring
* Hibernate where support is reliable

## 9.5 Firmware

Use a hidden firmware-update service such as `fwupd`.

Users should see:

* System update
* Device firmware update
* Restart required
* Update succeeded or failed

They should not have to understand LVFS or firmware package names.

---

# 10. Image construction

## 10.1 Prototype environment

Use Debian 13 as:

* Development host
* Build workstation
* Temporary runtime platform
* Hardware test platform
* Early package source
* QEMU/KVM host

Debian is not the intended public product base.

## 10.2 Production image

Use Yocto for the main product image.

Reasons:

* Controlled system composition
* Reproducible builds
* Architecture-specific layers
* SDK generation
* Security metadata
* Release layers
* Complete image generation
* Automated testing
* Long-term maintainability
* No requirement to expose a traditional package-based distribution

Buildroot may be used for early experiments or smaller appliance images.

Recommended structure:

```text
Development host: Debian 13
Prototype system: Debian dedicated session
Early image experiments: Buildroot if useful
Production image: Yocto LTS
```

---

# 11. Compositor strategy

## 11.1 Initial route

Use wlroots.

Reasons:

* Faster initial development
* Mature Wayland building blocks
* Modern display support
* Existing protocol support
* Multi-monitor capability
* Input support
* XWayland compatibility
* Reduced effort compared with implementing the graphics foundation

## 11.2 Smithay position

Smithay should remain a potential later alternative.

The architecture should isolate compositor-specific behavior behind internal interfaces so a future Smithay implementation is possible.

Do not maintain wlroots and Smithay implementations simultaneously during early development.

## 11.3 Prototype compositor

During early phases, run the Waystone graphical components on an existing wlroots compositor.

The existing compositor is scaffolding only and must not define the public product.

## 11.4 Dedicated compositor

Develop a custom wlroots compositor after core workflows work.

Initial requirements:

* XDG application windows
* Layer-shell surfaces
* Multiple monitors
* output detection
* resolution changes
* refresh-rate selection
* scaling
* keyboard input
* pointer input
* touchpad support
* move
* resize
* maximize
* fullscreen
* session locking
* clipboard
* drag and drop
* screen blanking
* hot-plugging
* docking and undocking
* XWayland compatibility
* four default workspaces

Explicitly exclude initially:

* Fancy animations
* blur
* translucency effects
* complex tiling
* compositor plugins
* scriptable window rules
* arbitrary visual themes
* desktop widgets
* wallpaper suites

---

# 12. Graphical workspace

The graphical interface should be called the:

> Waystone Workspace

Avoid calling it the shell, since “shell” will refer to the command interpreter.

## 12.1 Primary activities

* Explore
* Write
* Listen
* Record
* Publish
* Host
* Connect
* Learn
* Operate

## 12.2 Default workspaces

### Workspace 1: Explore

* Waystone Browser
* Feed reader
* Library
* Audio playback

### Workspace 2: Create

* Text editor
* Project files
* Waystone Helm
* Audio recording
* Audio editing
* Local preview

### Workspace 3: Publish

* Validation
* Transfer state
* Feed generation
* Publication comparison
* Publication history
* Remote verification

### Workspace 4: Operate

* Waystone Comm
* Local terminal
* SSH
* Hosts
* Services
* Logs
* Certificates
* Diagnostics

## 12.3 Interface structure

Potential layout:

```text
┌──────────────────────────────────────────────────────────────┐
│ System  Project  Publish  Network  Audio  Window  Help       │
├───────────────┬──────────────────────────────────────────────┤
│ Explore       │                                              │
│ Write         │               Active Workspace               │
│ Listen        │                                              │
│ Record        │                                              │
│ Publish       │                                              │
│ Host          │                                              │
│ Connect       │                                              │
│ Learn         │                                              │
│───────────────│                                              │
│ Hosts         │                                              │
│ Services      │                                              │
│ Transfers     │                                              │
│ Terminal      │                                              │
├───────────────┴──────────────────────────────────────────────┤
│ Create   Audio: Idle   Network: Connected   Battery: 82%     │
└──────────────────────────────────────────────────────────────┘
```

## 12.4 Visual requirements

* Compact rectangular controls
* visible borders
* clear menu hierarchy
* modest spacing
* readable typography
* fixed-width text for addresses and logs
* no hidden hover-only functions
* limited animation
* clear focus indicators
* keyboard accelerators
* high-contrast option
* HiDPI support
* reduced-motion option
* color-independent status indicators

---

# 13. Command-line environment

## 13.1 Terminal application

Waystone Comm should become the native terminal and communications application.

During early development, it may embed or wrap a mature terminal engine.

Required terminal features:

* Unicode
* 24-bit color
* searchable scrollback
* copy and paste
* address recognition
* tabs or multiple windows
* local and remote session labels
* keyboard font scaling
* accessibility
* security-state display
* session logging
* split views later

Examples:

```text
LOCAL — Project Console
REMOTE — offgridholdout.org
SERVICE — Gemini Activity
AUDIO — Encoder Log
```

## 13.2 Command shell

Use:

* Dash or another POSIX shell for scripts
* Bash for ordinary interactive use
* Optional Zsh in Workshop Mode

Do not create a new command interpreter for the first release.

## 13.3 Native commands

Top-level commands:

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

Examples:

```text
project list
project open long-century
publish --dry-run
service start gemini
record start
listen now
connect offgridholdout
certificate inspect capsule.example
network diagnose
update check
```

## 13.4 Command standards

Every Waystone command should support:

* Human-readable output
* JSON output
* useful exit codes
* `--help`
* noninteractive operation
* safe defaults
* dry-run mode
* explicit destructive confirmations
* consistent terminology

## 13.5 Workshop Mode

Workshop Mode exposes:

* Compiler toolchains
* SDK
* source code
* debugging
* package recipes
* Yocto tools
* build containers
* low-level logs
* test compositor sessions
* hardware diagnostics
* advanced filesystem access

Workshop Mode should use a separate development workspace rather than mutate the immutable system image.

---

# 14. Waystone shared platform

## 14.1 Waystone Core

Shared services and libraries:

* Project API
* Host API
* Identity API
* Publication API
* Library API
* Audio API
* Protocol dispatcher
* Configuration
* Notification model
* Permissions

## 14.2 Waystone UI

Shared visual components:

* Compact controls
* menus
* dialogs
* status indicators
* typography
* icon set
* keyboard behavior
* accessibility behavior
* workstation-style window standards

## 14.3 Waystone CLI

Shared command conventions:

* Output formatting
* JSON output
* authentication prompts
* errors
* progress reporting
* dry-run behavior
* confirmation prompts

---

# 15. Domain model

## 15.1 Project

Project types:

* Capsule
* Gemlog
* Gopherhole
* Spartan site
* Audio series
* Feed
* Pubnix home
* Documentation archive
* Classroom assignment
* Mixed publication

## 15.2 Publication

Properties:

* Project
* Destination
* Date
* Manifest
* Changed files
* Deleted files
* Validation results
* Content hashes
* Remote URL
* Rollback information

## 15.3 Host

Properties:

* Name
* Address
* Protocol
* SSH host key
* identities
* services
* publication destinations
* trust state
* notes

## 15.4 Identity

Properties:

* Display name
* SSH keys
* Gemini client certificates
* server certificates
* feed identity
* author metadata
* associated hosts
* expiration dates

## 15.5 Service

Types:

* Gemini
* Gopher
* Spartan
* Nex
* Finger
* SSH
* Feed host
* Audio stream
* Local preview
* Classroom directory

## 15.6 Feed

Types:

* RSS
* Atom
* Gemsub
* Twtxt
* Audio enclosure feed

## 15.7 Library item

Types:

* Saved page
* Bookmark
* Feed entry
* Audio
* Image
* Archive
* Local mirror
* Download

---

# 16. Project format

## 16.1 Structure

```text
example.wayproject/
├── project.toml
├── content/
│   ├── index.gmi
│   ├── about.gmi
│   └── gemlog/
├── audio/
│   ├── masters/
│   ├── published/
│   └── metadata/
├── assets/
├── feeds/
│   ├── feed.xml
│   └── gemsub.gmi
├── templates/
├── publish/
│   ├── staging.toml
│   └── production.toml
├── cache/
└── history/
```

## 16.2 Example manifest

```toml
[project]
name = "My Capsule"
type = "capsule"
language = "en"
author = "Nick"

[content]
root = "content"
index = "index.gmi"

[audio]
master_format = "flac"
publish_format = "opus"
publish_bitrate = 96000

[feed]
enabled = true
type = "atom"
path = "feeds/feed.xml"

[publish.production]
method = "rsync"
host = "offgridholdout"
remote_path = "/srv/gemini/nick"
url = "gemini://example.org"
```

Project formats must be versioned from the beginning.

---

# 17. Protocol support

## Tier 1: Stable-release requirements

* Gemini
* Gopher
* SSH
* SCP
* SFTP
* RSS
* Atom
* local files

## Tier 2: Intended for version 1 where reliable

* Spartan
* Nex
* Finger
* Twtxt
* Gemsub
* Titan

## Tier 3: Experimental or post-1.0

* Guppy
* NNTP
* IRC integration
* independent radio protocols
* Tor or I2P integration
* protocol bridges

## Common URI dispatcher

```text
gemini://
gopher://
spartan://
nex://
finger://
ssh://
sftp://
file://
```

The dispatcher determines:

* Which activity opens
* What identity applies
* What permissions are granted
* Whether a certificate is needed
* Whether content can be saved
* Whether compatibility mode is required

---

# 18. Waystone Browser integration

## 18.1 WaystoneOS profile

The WaystoneOS profile should emphasize:

* Gemini
* Gopher
* Spartan
* Nex
* Finger
* local files
* feeds
* project previews
* restricted HTTP

## 18.2 HTTP policy

The standalone Waystone Browser may continue supporting general HTTP and HTTPS.

The WaystoneOS profile should restrict HTTP use.

Allowed cases:

* Captive portals
* Project documentation
* Explicitly imported pages
* Repository access
* User-approved individual addresses
* Conventional-web discovery where unavoidable

WaystoneOS should not become a commercial-web workstation through Waystone Browser.

## 18.3 Native integrations

* Save pages to Waystone Library
* Open project previews
* Select client certificates
* Send audio to Listen
* Send terminal links to Waystone Comm
* Discover feeds
* display protocol information
* display certificate information
* validate local publications

---

# 19. Waystone Helm integration

## 19.1 Responsibility boundary

WaystoneOS Project Service:

* Knows project structure
* opens projects
* validates projects
* manages publication destinations
* tracks files
* tracks publication state

Waystone Helm:

* Knows goals
* tracks tasks
* tracks milestones
* shows project health
* identifies next actions
* organizes multiple projects
* reports momentum

## 19.2 Native project information

Helm can display:

* Unpublished changes
* Last publication
* Broken links
* Feed status
* Certificate status
* Audio items awaiting export
* Upcoming project milestone
* Suggested next action

Helm should be included but not forced upon every user.

---

# 20. Waystone Comm integration

Waystone Comm will provide:

* Local terminals
* SSH
* Telnet
* RLogin
* Serial
* ANSI BBS
* Pubnix access
* Host bookmarks
* Session logs
* host fingerprint information
* project-aware remote sessions
* service consoles
* file-transfer shortcuts
* local and remote visual distinction
* Finger and potentially IRC support

Comm should be integrated early because SSH publishing and pubnix use are fundamental workflows.

---

# 21. Writing environment

## 21.1 Editor requirements

* Gemtext syntax
* Plain text
* Markdown where useful
* Gophermap mode
* Feed-entry metadata
* Unicode
* spell checking
* word count
* line and column
* project navigation
* link insertion
* relative-link completion
* heading navigation
* templates
* autosave
* snapshots
* preview
* validation
* distraction-free mode

## 21.2 Exclusions

* Rich-text editing
* word-processing pagination
* WYSIWYG web design
* complex page layout
* collaborative cloud editing
* citation-management suite
* plugin marketplace
* built-in AI writing features in the initial release

---

# 22. Audio system

## 22.1 Infrastructure

Use:

* PipeWire
* WirePlumber
* ALSA
* GStreamer or FFmpeg libraries

Normal users should not need to know these implementation names.

## 22.2 Listen activity

Features:

* Local playback
* remote audio
* feed enclosures
* internet radio
* playlists
* queue
* resume
* playback speed
* downloads
* metadata
* output selection
* keyboard media controls
* project audio preview

Formats:

* Opus
* Ogg Vorbis
* FLAC
* WAV
* MP3
* AAC where practical

## 22.3 Record activity

Features:

* Microphone selection
* level meter
* clipping warning
* mono or stereo
* WAV master
* FLAC master
* direct Opus recording where useful
* monitoring
* pause
* markers
* automatic naming
* project attachment
* interrupted-recording recovery

## 22.4 Editing

Version 1:

* Trim
* split
* join
* fade in
* fade out
* normalize
* DC offset removal
* channel conversion
* sample-rate conversion
* metadata
* export presets

## 22.5 Publishing presets

```text
Voice — compact
Opus, mono, 48 kbps

Voice — standard
Opus, mono, 64 kbps

Spoken program
Opus, stereo, 96 kbps

Music — efficient
Opus, stereo, 128 kbps

Music — quality
Opus, stereo, 192 kbps

Archive
FLAC
```

## 22.6 Audio hardware testing

Test:

* Built-in laptop audio
* headphones
* analog microphone
* USB headset
* USB audio interface
* Bluetooth headset
* HDMI audio
* DisplayPort audio
* hot-plugging
* suspend and resume
* volume keys
* mute keys

## 22.7 Standard and Studio views

Standard:

* Device
* volume
* microphone
* record
* playback
* export

Studio:

* Routing
* channel selection
* sample rate
* buffers
* latency
* monitoring
* multiple devices
* JACK compatibility

---

# 23. Publishing

## 23.1 Methods

Priority:

1. Rsync over SSH
2. SCP
3. SFTP
4. Titan
5. Git
6. Local-service publication
7. Removable-media export

## 23.2 Stages

```text
Prepare
   ↓
Validate
   ↓
Compare
   ↓
Preview
   ↓
Transfer
   ↓
Verify remotely
   ↓
Record publication
```

## 23.3 Validation

Check:

* Gemtext structure
* internal links
* missing files
* address syntax
* duplicate feed IDs
* malformed RSS or Atom
* MIME types
* oversized audio
* unsupported characters
* permissions
* certificates
* remote quota
* destination availability

## 23.4 History

Record:

* Date
* project version
* destination
* changes
* deletions
* hashes
* transfer result
* verification
* identity
* rollback data

---

# 24. Hosting

## 24.1 Local services

* Gemini
* Gopher
* Spartan
* Nex
* Finger
* SSH
* local preview
* feeds
* audio resources
* classroom directory

## 24.2 Workflow

1. Create service.
2. choose project.
3. choose access scope.
4. choose identity.
5. select port.
6. validate.
7. start service.
8. test locally.
9. optionally test externally.

## 24.3 Access profiles

* This computer only
* Local network
* VPN
* Public internet
* Custom

## 24.4 Security defaults

* Disabled until configured
* least privilege
* separate service users
* read-only content where possible
* service sandboxing
* generated firewall rules
* explicit public-exposure confirmation
* rate limiting
* SSH password login disabled by default
* certificate-expiry warnings

---

# 25. Identity and certificate management

## 25.1 Managed objects

* Gemini client certificates
* server certificates
* SSH keys
* SSH host fingerprints
* author identities
* feed identities
* signing keys
* publication credentials

## 25.2 Operations

```text
identity create
identity list
identity inspect
identity export
identity import
identity lock
identity remove

certificate create
certificate inspect
certificate renew
certificate export
certificate revoke
certificate trust
```

## 25.3 Security

* Credentials in encrypted workspace
* separate encryption where appropriate
* warnings before export
* credentials excluded from ordinary project exports
* recovery documentation
* expiration warnings
* hardware-backed keys later

---

# 26. User-visible storage

```text
Workspace
├── Projects
│   ├── Capsules
│   ├── Gopherholes
│   ├── Audio Series
│   ├── Feeds
│   └── Classroom
├── Writing
│   ├── Drafts
│   ├── Notes
│   └── Templates
├── Audio
│   ├── Recordings
│   ├── Masters
│   ├── Published
│   ├── Downloads
│   └── Playlists
├── Library
│   ├── Bookmarks
│   ├── Feeds
│   ├── Saved Pages
│   └── Archives
├── Connections
│   ├── Hosts
│   ├── Identities
│   ├── Certificates
│   └── Destinations
├── Services
│   ├── Gemini
│   ├── Gopher
│   └── Other
└── Storage
    ├── Internal
    ├── Removable
    ├── Network
    └── Backups
```

The Unix filesystem should only be exposed in Workshop Mode or advanced diagnostics.

---

# 27. Boot and persistence

## 27.1 Normal boot

```text
WaystoneOS

Checking hardware
Opening system image
Preparing workspace
Starting network
Starting Waystone Workspace
```

Do not normally show:

* GRUB branding
* kernel logs
* systemd unit lists
* TTY login
* distribution branding

## 27.2 First boot

Options:

* Temporary Session
* Create Encrypted Workspace
* Open Existing Workspace
* Learn About WaystoneOS

## 27.3 Persistent storage

Suggested layout:

```text
EFI system partition
System image A
System image B
Recovery image
Encrypted workspace
Optional shared export partition
```

## 27.4 Session types

### Temporary

* RAM-backed
* no saved history
* no retained keys
* optional export before shutdown

### Persistent

* encrypted projects
* credentials
* feeds
* bookmarks
* recordings
* settings

### Demonstration

* sample capsule
* guided lessons
* local services
* automatic reset

### Classroom

* temporary identity
* teacher discovery
* controlled publishing
* assignment export

---

# 28. Updates and recovery

## 28.1 Update model

Use signed A/B image updates.

Process:

1. Check metadata.
2. download signed image.
3. verify signature and hash.
4. write inactive partition.
5. run checks.
6. reboot.
7. verify health.
8. mark successful.
9. revert automatically if necessary.

## 28.2 Channels

* Stable
* Preview
* Development

## 28.3 Recovery

Options:

* Start previous system version
* Check workspace
* Repair boot
* Reset display
* Reset network
* Export user data
* Reinstall system
* Open diagnostic console

---

# 29. Security architecture

## 29.1 Threats

* Stolen USB device
* Malicious downloaded content
* Untrusted servers
* compromised remote hosts
* hostile public Wi-Fi
* exposed services
* unsafe compatibility tools
* tampered updates
* credential leakage
* vulnerable media decoders
* malicious feeds
* Workshop Mode changes

## 29.2 Controls

* LUKS encryption
* signed images
* read-only base
* sandboxing
* minimum permissions
* explicit public exposure
* nftables
* separated identities
* host-key verification
* certificate inspection
* no removable-media autorun
* no execution of downloaded content
* isolated media decoding where practical
* update verification
* fixed recovery image

---

# 30. Accessibility

Version 1 baseline:

* Full keyboard use
* visible focus
* interface scaling
* font scaling
* high contrast
* reduced motion
* screen-reader compatibility
* semantic labels
* configurable audio alerts
* transcript attachment
* playback-speed controls
* mono-audio option
* keyboard media controls
* color-independent state indicators

---

# 31. Implementation stack

## Compositor

* C
* wlroots

## Core services and CLI

* Rust

## Applications

* Qt 6 initially
* C++ or Rust Qt bindings after prototyping

## Audio

* PipeWire
* WirePlumber
* GStreamer or FFmpeg

## IPC

* D-Bus for most service interaction
* private Unix sockets where high-volume transfer requires them

## System construction

* Debian 13 development host
* Yocto production image

## Updates

* A/B image system
* RAUC, OSTree, systemd-sysupdate, or equivalent after evaluation

---

# 32. Internal services

```text
waystone-projectd
waystone-publishd
waystone-identityd
waystone-hostd
waystone-serviced
waystone-audiod
waystone-libraryd
waystone-updated
waystone-sessiond
waystone-hardwared
```

Each service should have:

* Narrow privileges
* versioned API
* structured logs
* unit tests
* CLI test client
* documented failure behavior

---

# 33. Repository structure

## Product repositories

```text
waystone/
├── waystone-os
├── waystone-browser
├── waystone-helm
├── waystone-comm
└── waystone-site
```

## WaystoneOS repository

```text
waystone-os/
├── docs/
├── compositor/
├── workspace/
├── apps/
├── services/
├── cli/
├── integration/
│   ├── browser/
│   ├── helm/
│   └── comm/
├── protocols/
├── project-format/
├── packaging/
├── yocto/
│   └── meta-waystone/
├── images/
├── installer/
├── recovery/
├── tests/
├── tools/
└── examples/
```

Use a monorepo for WaystoneOS during the first year.

---

# 34. Development phases

## Phase 0: Charter

Deliverables:

* Product charter
* final non-goals
* terminology
* personas
* workflow maps
* security model
* version 0.1 scope
* licensing policy
* repositories
* architecture decision records

Exit criterion:

The project has a stable definition and release boundary.

## Phase 1: Graphical prototype

Build on Debian with Qt.

Deliverables:

* Four workspaces
* activity navigation
* project view
* hosts
* services
* audio status
* Waystone Browser launch
* Waystone Comm launch
* Helm project view
* settings mockup

Exit criterion:

Users can understand the operating model without seeing a normal Linux desktop.

## Phase 2: Core services

Build:

* project service
* publication service
* identity service
* host service
* CLI framework
* D-Bus interfaces
* configuration schema
* logs

Exit criterion:

GUI and CLI operate on the same project data.

## Phase 3: Text publication vertical slice

Workflow:

1. Create capsule.
2. write Gemtext.
3. preview.
4. validate.
5. configure SSH target.
6. publish.
7. verify remotely.
8. record history.

Exit criterion:

A real capsule can be published entirely through WaystoneOS.

## Phase 4: Audio vertical slice

Workflow:

1. Record.
2. trim.
3. normalize.
4. export Opus.
5. attach.
6. update feed.
7. publish.
8. play remote result.

Exit criterion:

A complete audio entry can be published without external production software.

## Phase 5: Bootable image

Deliverables:

* Yocto layer
* kernel configuration
* firmware
* boot presentation
* Wayland session
* graphical workspace
* networking
* audio
* persistence
* recovery console

Exit criterion:

USB image boots on three reference systems and completes text and audio workflows.

## Phase 6: Encrypted persistence

Deliverables:

* LUKS workspace
* first-run setup
* temporary mode
* persistent mode
* backup
* recovery key
* clean shutdown

Exit criterion:

A workspace can move safely between reference machines.

## Phase 7: Custom compositor

Deliverables:

* wlroots compositor
* displays
* input
* workspaces
* decorations
* locking
* XWayland
* multi-monitor
* crash recovery

Exit criterion:

All first-party applications function reliably without the prototype compositor.

## Phase 8: Hosting

Deliverables:

* Gemini
* Gopher
* certificate creation
* access profiles
* firewall integration
* logs
* tests

Exit criterion:

Users can host locally without editing raw service configuration.

## Phase 9: Updates and recovery

Deliverables:

* A/B images
* signatures
* health checks
* rollback
* recovery image
* export tools
* update channels

Exit criterion:

A failed update automatically returns to a functioning system.

## Phase 10: Installation

Deliverables:

* disk selection
* safeguards
* partitioning
* encryption
* UEFI setup
* recovery
* reinstall preserving workspace

## Phase 11: Classroom mode

Deliverables:

* teacher node
* student discovery
* temporary names
* local directory
* assignment templates
* local publication
* export and reset

## Phase 12: Public alpha

Deliverables:

* Hardware report tool
* compatibility submissions
* public images
* issue templates
* release notes
* known limitations
* documentation

---

# 35. Testing

## Unit tests

* Manifests
* feeds
* URIs
* certificate metadata
* publication comparison
* audio metadata
* update signatures
* configuration migration

## Integration tests

* GUI to service
* CLI to service
* project to publication
* recording to export
* service creation to startup
* workspace unlock
* update rollback

## QEMU tests

Automate:

* boot
* first-run
* graphical session
* project creation
* publishing
* shutdown
* update
* rollback

## Hardware testing

Record:

* Boot
* graphics
* scaling
* Wi-Fi
* Ethernet
* Bluetooth
* audio output
* microphone
* USB audio
* suspend
* resume
* battery
* brightness
* touchpad
* external monitor
* removable storage
* update
* recovery

## Reference machines

At minimum:

1. Modern AMD desktop
2. Modern Intel laptop
3. Older Intel laptop
4. AMD laptop
5. Low-resource x86-64 thin client
6. USB-audio test system
7. QEMU target
8. NVIDIA target later

---

# 36. Performance targets

## Minimum target hardware

* x86-64 dual-core CPU
* 4 GB RAM
* 16 GB storage
* Intel or AMD graphics
* 1366×768 display
* Ethernet or supported Wi-Fi

## Preferred hardware

* Four-core CPU
* 8 GB RAM
* 32 GB storage
* 1920×1080 display
* class-compliant audio

## Goals

* Boot within 30 seconds from SSD-class media
* Graphical idle memory below approximately 1.5 GB
* Terminal launch below one second
* Editor launch below two seconds after warm start
* Smooth basic browsing with 4 GB RAM
* No audio dropouts under ordinary load
* Local preview below one second
* Reliable suspend and resume
* Base image ideally under 4 GB

---

# 37. Documentation

## User

* What the small web is
* Booting
* creating a workspace
* networking
* browsing
* writing Gemtext
* recording
* publishing
* hosting
* pubnix access
* backups
* updates
* recovery
* Workshop Mode

## Administrator

* Service exposure
* firewall behavior
* certificates
* remote hosts
* logs
* classroom mode
* deployment

## Developer

* Build setup
* repositories
* APIs
* project format
* compositor
* UI standards
* testing
* Yocto
* releases

Documentation should be available:

* Locally
* Over Gemini
* Over Gopher
* On a conventional website
* As downloadable text archives

---

# 38. Major risks

## Generic desktop drift

Mitigation:

Every proposed feature must identify a direct small-web workflow.

## Compositor consumes the project

Mitigation:

Complete publishing workflows before building the custom compositor.

## Audio becomes a DAW

Mitigation:

Limit version 1 to recording, basic editing, encoding, metadata, feeds, and publishing.

## Hardware support becomes unmanageable

Mitigation:

Start with x86-64 Intel and AMD systems and publish compatibility limits.

## Small-web applications become abandoned

Mitigation:

Put applications behind stable Waystone activities and services.

## Yocto complexity slows development

Mitigation:

Prototype on Debian first.

## Experts resent restrictions

Mitigation:

Provide a capable normal CLI and Workshop Mode.

## Solaris influence becomes nostalgia

Mitigation:

Use functional workstation principles rather than copying historical visuals exactly.

## Scope exceeds available developers

Mitigation:

Build vertical slices and use mature components before replacing them.

---

# 39. First 90-day plan

## Weeks 1–2

* Create WaystoneOS repository
* Finalize charter
* document non-goals
* establish terminology
* define project format
* define CLI
* establish product-family interfaces
* write architecture records
* define version 0.1

## Weeks 3–4

* Build Qt workspace frame
* implement four workspaces
* add navigation
* add project mock data
* add host mock data
* add service mock data
* add audio state
* launch Browser, Helm, and Comm prototypes

## Weeks 5–6

* Build project service
* define D-Bus API
* implement project create
* implement project list
* implement project inspect
* implement project validate
* connect project GUI
* create sample capsule

## Weeks 7–8

* Integrate text editor
* add Gemtext support
* project navigation
* local preview
* link validation
* snapshots
* complete local authoring workflow

## Weeks 9–10

* Build publication service
* create host model
* implement Rsync over SSH
* add dry run
* add remote verification
* publication history
* GUI and CLI parity
* Waystone Comm integration

## Weeks 11–12

* Enumerate PipeWire devices
* basic playback
* microphone selection
* WAV or FLAC recording
* Opus export
* project attachment
* feed enclosure
* audio publication
* playback through Waystone Browser or Listen

## Ninety-day success condition

The Debian-hosted development session completes:

```text
Open project in Waystone Helm
        ↓
Write Gemtext
        ↓
Record audio
        ↓
Export Opus
        ↓
Attach recording
        ↓
Generate feed
        ↓
Publish over SSH
        ↓
Inspect host in Waystone Comm
        ↓
Open result in Waystone Browser
```

---

# 40. Provisional architecture summary

| Area                 | Decision                                               |
| -------------------- | ------------------------------------------------------ |
| Product name         | WaystoneOS                                             |
| Product family       | Browser, Helm, Comm, OS                                |
| First release        | Persistent x86-64 live USB                             |
| Installable system   | Added after technical preview                          |
| Kernel               | Linux LTS                                              |
| Prototype host       | Debian 13                                              |
| Production image     | Yocto LTS                                              |
| Prototype compositor | Existing wlroots compositor                            |
| Product compositor   | Custom wlroots compositor                              |
| Smithay              | Possible later alternative                             |
| Graphical toolkit    | Qt 6 initially                                         |
| Core services        | Rust                                                   |
| Compositor language  | C                                                      |
| Audio                | PipeWire and WirePlumber                               |
| CLI                  | Bash plus native Waystone commands                     |
| Package manager      | Hidden and unavailable in normal mode                  |
| Workshop Mode        | Separate development environment                       |
| General browser      | No Firefox or Chromium                                 |
| HTTP                 | Restricted Waystone Browser profile and captive portal |
| Update model         | Signed A/B images                                      |
| Visual model         | Functional Solaris/CDE/OpenWindows influence           |
| First architecture   | x86-64                                                 |
| Later architecture   | ARM64 after stable x86-64 release                      |
| Primary workflow     | Explore, create, publish, connect, operate             |
| Audio scope          | Playback, recording, editing, feeds, publishing        |

# Final project definition

WaystoneOS will not be a lightweight Linux distribution with packages removed.

It will be a complete, portable small-web workstation built around independent places, publications, identities, hosts, recordings, and connections.

Linux will provide the hardware foundation.

WaystoneOS will provide the operating system the user actually experiences.

The next planning artifact should be the **Phase 0 project charter and architecture-decision register**, which would convert this master plan into the first concrete development documents.
