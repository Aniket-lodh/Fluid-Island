---
name: testing-and-debugging
description: Unit and integration tests, crash recovery, and reproducible diagnostics for Fluid Island. Use when adding tests, reproducing bugs, or validating behavior.
license: AGPL-3.0-only
compatibility: opencode
---

## What I do

- Define testing strategy across Rust core, IPC, and extension.
- Provide reproducible diagnostic workflows.
- Cover crash recovery and failure injection scenarios.

## When to use me

Use for test authoring and debugging. For performance measurement, use
`performance-profiling`.

## Test layers

- Unit tests: state machine, parsing, config, pure logic.
- Integration tests: D-Bus contract, core lifecycle, fallback window.
- Extension tests: nested shell harness (isolated).
- Manual: accessibility, visual, reduced-motion.

## Crash recovery scenarios (must pass before release)

- Core crashes while extension attached -> extension resets collapsed; on
  restart, `Attach()` resyncs.
- Core hangs -> clicks still respond within 500 ms; late reply applied.
- Shell restarts -> extension re-enables; core exits cleanly; restart core.
- Core absent -> extension local expand/collapse works.
- Fullscreen toggle -> island hides/restores.

## Reproducible diagnostics

- Capture logs with timestamps and environment details.
- Use isolated HOME/XDG and private session bus for shell tests.
- Record exact commands and expected vs. actual outcomes.
- Save screenshots for UI regressions.

## Commands

```sh
cargo test
cargo test -- --nocapture
# Nested shell tests
./spikes/gnome-nested/tests/run-all.sh
```

## Common mistakes

- Claiming tests passed without running them.
- Testing against the real session.
- Leaving stray processes (use idempotent control scripts).
- Non-deterministic timing assumptions in tests.

## Authoritative documentation

- Rust testing: https://doc.rust-lang.org/book/ch11-00-testing.html
- cargo test: https://doc.rust-lang.org/cargo/commands/cargo-test.html
- GJS testing: https://gjs.guide/extensions/development/debugging.html
