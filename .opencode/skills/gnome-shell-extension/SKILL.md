---
name: gnome-shell-extension
description: GNOME Shell extension development for Fluid Island: panel integration, lifecycle, fullscreen behavior, and GNOME 48 compatibility. Use when editing extension JS/metadata or integration tests.
license: AGPL-3.0-only
compatibility: opencode
---

## What I do

- Define the thin-extension contract for Fluid Island.
- Cover lifecycle, fullscreen handling, and state synchronization.
- Provide testing workflow using the nested GNOME Shell harness.
- List compatibility and maintenance constraints.

## When to use me

Use for the optional GNOME Shell extension. Do not put business logic in the
extension; the core owns state.

## Extension contract

- Render the pill in the top bar; place to the right of the clock.
- Forward clicks to core via D-Bus `Toggle()` with ~500 ms timeout.
- Mirror core state via `StateChanged` signal.
- On core vanish: reset to collapsed.
- On core return: call `Attach()` and apply returned state.
- On disable: send `Detach()`, remove actor, restore fallback.

## Metadata

- Pin `shell-version` to `["48"]` until compatibility is explicitly widened.
- Keep `uuid` stable: `island-spike@fluid-island.local` is a prototype name;
  choose a final UUID before release.

## Fullscreen

- Use `addTopChrome(..., { trackFullscreen: true })` so the pill hides during
  fullscreen and restores afterwards.

## State synchronization rules

- Core always starts collapsed.
- `Toggle()` and `Attach()` return authoritative state.
- Extension D-Bus calls must have timeouts.
- Clicks must still respond locally when core is unavailable/hung.
- Late replies from hung cores are applied when they arrive.

## Testing (isolated only)

Use `spikes/gnome-nested/` harness:

```sh
# Start nested shell (private bus, redirected HOME/XDG)
./spikes/gnome-nested/run-nested.sh
# Sync spike sources
./spikes/gnome-nested/sync-runtime.sh
# Run tests
./spikes/gnome-nested/tests/run-all.sh
# Stop nested instance only
./spikes/gnome-nested/stop-nested.sh
```

Never touch the real session. Never install the extension into the user's real
GNOME without approval.

## Common mistakes

- Holding business logic in the extension.
- Covering the panel clock with the collapsed pill.
- Forgetting to reset state on core vanish.
- Not handling shell restarts (extension re-enables; core exits).
- Using `global.log` (not a function in GNOME 48).
- Importing internal modules without version pinning.

## Authoritative documentation

- GNOME Shell extensions: https://gjs.guide/extensions/
- GNOME Shell source (48): https://gitlab.gnome.org/GNOME/gnome-shell
- GJS guide: https://gjs.guide/
- Mutter/GNOME compositor: https://mutter.gnome.org/
