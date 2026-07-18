# ADR-0002: Prototype on Debian Before Yocto Image Construction

Status: Accepted
Date: 2026-07-17

## Context

WaystoneOS will eventually need a controlled production image. Yocto is a strong fit for reproducibility, image composition, SDK generation, license manifests, SBOMs, CVE metadata, and release engineering.

Starting with Yocto too early would slow validation of the product model, UI, services, CLI, and workflows.

## Decision

Use Debian 13 as the development host, prototype environment, hardware test platform, and early runtime platform.

Use Yocto for the production WaystoneOS image after useful vertical slices work.

The version 0.1 development preview runs as a dedicated Wayland session on Debian. It is not the final product base or public identity.

## Consequences

- Early development can focus on workflow validation.
- Existing Debian packaging and tooling can support rapid prototyping.
- Yocto work is deferred until the product shape is proven.
- Technical documentation must avoid describing Debian as the WaystoneOS product base.
- Any prototype packaging must be treated as scaffolding.

## Alternatives Considered

- Start immediately with Yocto: rejected because it risks build-system complexity before user workflows are proven.
- Ship a Debian remix: rejected because it conflicts with product identity and normal-mode constraints.
- Use Buildroot for the main product: not selected for the main path, but may be useful for experiments or smaller appliance images.

## Follow-Up

- Define the Debian development session layout for version 0.1.
- Record the future Yocto layer name and image goals when image work begins.
- Avoid public claims about production image behavior until booted and tested.

