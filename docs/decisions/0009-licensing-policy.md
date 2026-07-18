# ADR-0009: Licensing Policy

Status: Accepted
Date: 2026-07-17

## Context

The project needs a licensing policy before public implementation work begins. The plan identifies licensing policy as a Phase 0 deliverable.

WaystoneOS should be easy to inspect, reuse, package, study, and adapt without creating avoidable legal friction. A permissive license matches the project's practical small-systems culture.

## Decision

WaystoneOS uses the MIT License as the project-wide default license.

The root [LICENSE](../../LICENSE) file contains the license text.

This default applies to source code and project documentation unless a file or directory states a different license.

Original project assets should use MIT by default unless a later asset-specific policy is adopted. Third-party assets, protocol fixtures, fonts, icons, media samples, firmware references, package metadata, and vendor code must preserve their own licenses and attribution requirements.

## Consequences

- Implementation work can proceed under a clear default license.
- Documentation can be copied and redistributed with the project.
- Third-party material still requires explicit license review before inclusion.
- Future packaging must preserve license notices and attribution.
- If another license is needed for a specific component, that exception must be documented.

## Alternatives Considered

- Apache-2.0: reasonable and patent-explicit, but more complex than needed for the project default.
- GPL family: rejected as the default because the project currently favors broad reuse and low packaging friction.
- Separate documentation license: deferred; MIT is acceptable for the current planning and developer documentation.

## Follow-Up

- Add contribution guidance before accepting public contributions.
- Add third-party license tracking before importing dependencies or assets.
- Revisit asset licensing if the project grows a distinct icon, sound, or documentation corpus that needs different terms.
