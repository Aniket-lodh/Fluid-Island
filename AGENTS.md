# Fluid Island — Agent Operating Rules

Fluid Island is an early-stage open-source project exploring an adaptive, glanceable
desktop surface for Linux. This document governs how coding agents work in this
repository. Read it fully before making changes.

## 1. Product goals and architecture

### Product goals

- Provide a small, quiet, unobtrusive desktop presence that expands only when
  something needs attention, presents useful controls/information, then settles
  back.
- Represent many kinds of activity (media, notifications, downloads, timers,
  connectivity, and future integrations) rather than a single feature.
- Be distinctly Linux-native: desktop integration and compatibility matter as
  much as appearance.
- Prioritize reliability, performance, accessibility, predictable behavior, and
  easy recovery over visual effects.
- Be quiet by default; motion communicates state changes and stays responsive.
- No telemetry, network services, background collection, or external dependencies
  without documented need and privacy implications.

### Architecture (intended)

- **Core (Rust + GTK4/libadwaita):** owns state and UI. This is the source of
  truth for island state and behavior.
- **Optional GNOME Shell extension (JavaScript/GJS):** a thin rendering/integration
  layer that places the pill in the top bar, forwards clicks, mirrors core state.
  It must never hold business logic.
- **D-Bus session service:** communication between core and extension.
  Methods: `Toggle()`, `Attach()`, `Detach()`, `GetState()`. Signal:
  `StateChanged(state)`. Core always starts collapsed.
- **Fallback:** when no extension is attached, the core shows a normal window with
  working controls.
- Extension is optional; the app must remain usable without it.

### Key design findings (from spikes in `spikes/` and `audit/`)

- Plain GTK4 toplevels cannot choose their own position; the compositor places
  them. Top-bar placement requires the GNOME extension.
- The collapsed pill must not cover the panel clock; place it to the right of
  the clock and re-place when the clock geometry changes.
- Expansion grows away from the clock; clamp to screen bounds.
- Hide the island during fullscreen windows; restore afterwards.
- Extension must reset to collapsed when the core vanishes and resync via
  `Attach()` when the core returns.
- All D-Bus calls need timeouts (~500 ms); clicks must still respond locally
  when the core is unavailable or hung.
- Extension imports internal GNOME JS modules; pin to a specific shell version.

## 2. Environment and compatibility

- Primary development environment: **Debian 13**.
- Initial compatibility target: **GNOME 48** (Shell 48.x). Pin extension
  `shell-version` to 48 until explicitly widened.
- Real session must remain untouched during development/testing. Use the nested
  GNOME Shell test harness in `spikes/gnome-nested/` (private session bus,
  redirected HOME/XDG/GSettings) for any shell-level testing.
- Do not modify, restart, or inspect the user's active desktop session without
  explicit approval.

## 3. Coding standards

- Language: Rust for the core; JavaScript/GJS for the optional extension.
- Follow idiomatic Rust: ownership/borrowing clarity, explicit error types,
  minimal `unsafe`, no panics in normal control flow.
- Modular design: small focused modules, clear boundaries between UI, state,
  IPC, and integrations.
- Error handling: return `Result` types; never silently ignore errors; log with
  context. Avoid `unwrap()`/`expect()` in production paths.
- Dependency management: justify every new crate/dependency with license and
  maintenance notes; prefer well-maintained, minimal dependencies; keep the
  dependency tree auditable.
- Formatting: `rustfmt` for Rust; follow `.editorconfig` for everything else.
- Linting: `clippy` with warnings as errors for new code where practical.
- Naming: clear, descriptive names; avoid abbreviations except common ones.
- Comments: explain why, not what; keep public APIs documented with rustdoc.

## 4. Required verification

Before claiming work is complete, run and report actual results:

- `cargo fmt --check`
- `cargo clippy` (or project-configured lint)
- `cargo test`
- Relevant integration/manual checks
- Performance checks for any animation/rendering change

Never claim a test passed unless it was executed successfully. If a check cannot
run, state why and what was not verified.

## 5. Accessibility, UI consistency, maintainability

- Respect keyboard navigation, focus order, and screen-reader semantics.
- Honor reduced-motion preferences; provide non-animated alternatives.
- Support high-contrast themes and scalable text.
- Keep spacing, motion, and interaction states consistent with the design system.
- Motion must be interruptible and must not block input.
- Maintain clear module ownership and avoid hidden global state.

## 6. Security and privacy

- Request only the permissions needed (process, IPC, filesystem, network).
- Validate all IPC inputs; treat the extension and any external caller as
  untrusted.
- No secrets, tokens, or personal data in the repository.
- No network activity without documented need and privacy review.
- Shell integration and plugin loading must be explicit and auditable.
- Filesystem access limited to required paths.

## 7. Development workflow and reporting

- Work in small, reviewable increments.
- Before substantial changes, open an issue or document the proposal.
- Preserve existing work; do not overwrite useful configuration or prototype files.
- Report what changed, how it was validated, and any remaining risks.
- Use isolated test environments; do not touch the host session.

## 8. Milestones (directional)

1. Repository and agent infrastructure (this phase).
2. Rust core scaffold with state machine and fallback window.
3. D-Bus service and contract tests.
4. Standalone application polish.
5. Linux desktop integrations.
6. Performance, accessibility, and reliability hardening.
7. Standalone application packaging and installation.
8. Optional GNOME Shell extension for GNOME 48.

## 9. Safety requirements (approval needed)

The agent must request explicit user approval before:

- Installing system packages or changing system configuration.
- Running privileged commands (`sudo`, etc.).
- Modifying or restarting the active desktop session.
- Enabling, disabling, or installing the GNOME extension in the real session.
- Creating or enabling persistent background services.
- Adding external plugins, MCP servers, or third-party skills that execute code
  or access external services.
- Performing destructive operations or changing permissions outside the project.

Use isolated test environments wherever possible. Never claim a test passed
unless it was executed successfully.

## 10. Repository notes

- `audit/` contains research and spike reports; treat as read-only reference.
- `spikes/` contains throwaway prototype code; do not build product code on it
  without review.
- `branding/` contains placeholder assets.
- There is no application code yet; this phase is infrastructure only.
