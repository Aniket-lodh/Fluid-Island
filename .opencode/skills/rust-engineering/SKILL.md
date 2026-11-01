---
name: rust-engineering
description: Safe, idiomatic, maintainable Rust for the Fluid Island core. Use when writing, reviewing, or refactoring Rust code, managing crates/dependencies, or designing error handling and module boundaries.
license: AGPL-3.0-only
compatibility: opencode
---

## What I do

- Guide idiomatic Rust implementation for the core application.
- Define module boundaries, error handling, and dependency practices.
- Provide commands for formatting, linting, testing, and auditing.
- Flag common Rust mistakes and unsafe patterns.

## When to use me

Use this skill when implementing or reviewing Rust code in the core. Do not use it
for GTK-specific UI details (see `gtk4-libadwaita`) or GNOME extension code.

## Core principles

- Prefer clarity over cleverness. Keep functions small and single-purpose.
- Make invalid states unrepresentable where practical with enums/newtypes.
- Avoid `unwrap()`/`expect()` in production paths; propagate with `Result`.
- Minimize `unsafe`; document any `unsafe` block with a safety comment.
- Keep I/O, state, and UI concerns separated.

## Module layout (suggested)

- `core/` state machine and domain logic
- `ui/` GTK widgets and views
- `ipc/` D-Bus client/server bindings
- `integrations/` activity providers (media, notifications, timers)
- `config/` configuration loading and validation

## Error handling

- Define a crate-level error enum (`thiserror` or manual) with context.
- Use `anyhow` only at application boundaries, not in library code.
- Log errors with `tracing` including relevant IDs; never silently drop.
- Convert foreign errors with `From` impls rather than ad-hoc strings.
- Distinguish recoverable vs. fatal errors; crash recovery must leave state clean.

## Dependency management

- Justify each new crate in the PR (purpose, license, maintenance).
- Prefer crates with active maintenance and permissive/compatible licenses.
- Run `cargo tree` and check for duplicates; avoid heavy transitive deps.
- Pin versions in `Cargo.lock` for applications; review updates deliberately.
- Audit with `cargo audit` when available.

## Commands

```sh
cargo fmt --check
cargo clippy --all-targets --all-features
cargo test
cargo build --release
cargo tree
cargo audit
```

## Common mistakes

- Using `unwrap()` on IPC or filesystem results.
- Holding GTK objects across threads without care.
- Blocking the main loop with synchronous I/O.
- Leaking tasks/timers; not canceling on teardown.
- Overusing `Arc<Mutex<...>>` instead of message passing.
- Ignoring `Send`/`Sync` requirements for async code.

## Authoritative documentation

- The Rust Book: https://doc.rust-lang.org/book/
- Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
- Clippy lints: https://rust-lang.github.io/rust-clippy/master/
- GTK4 Rust bindings: https://gtk-rs.org/gtk4-rs/stable/latest/docs/gtk4/
- libadwaita Rust: https://world.pages.gitlab.gnome.org/Rust/libadwaita-rs/
- D-Bus (zbus): https://docs.rs/zbus/latest/zbus/
