---
name: performance-profiling
description: CPU, memory, startup time, responsiveness, and animation performance for Fluid Island. Use when measuring, optimizing, or validating performance claims.
license: AGPL-3.0-only
compatibility: opencode
---

## What I do

- Define performance metrics and measurement methods.
- Provide profiling commands and tooling guidance.
- Set budgets and regression checks.

## When to use me

Use for any performance work or when a change touches rendering/animation/IPC.

## Metrics

- Startup time (cold/warm).
- Idle CPU and memory footprint.
- Animation frame time (target 60 fps, no jank).
- IPC round-trip latency (target < 5 ms).
- Responsiveness: click-to-visual feedback.

## Measurement methods

- Use `perf`, `heaptrack`, `valgrind` (massif) for native profiling.
- Use `tracing` + `tracing-chrome` for async spans.
- Measure shell idle cost via CPU sampling over fixed windows.
- Capture frame timings during animations.

## Commands

```sh
cargo build --release
/usr/bin/time -v ./target/release/fluid-island
perf record -g ./target/release/fluid-island
heaptrack ./target/release/fluid-island
```

## Budgets (initial, to be validated)

- Idle CPU: negligible (< 1% of one core average).
- Idle memory: minimal; document baseline.
- Animation: no frame drops during transitions.
- IPC: < 5 ms round trip on same machine.

## Common mistakes

- Claiming performance improvements without measurements.
- Profiling debug builds.
- Ignoring compositor/GPU effects on frame timing.
- Measuring only best case; include worst case (hung core, high load).

## Authoritative documentation

- perf: https://perf.wiki.kernel.org/
- heaptrack: https://github.com/KDE/heaptrack
- Rust performance book: https://nnethercote.github.io/perf-book/
- GTK rendering: https://docs.gtk.org/gtk4/overview.html
