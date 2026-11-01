---
name: desktop-ui-design
description: Visual consistency, interaction states, spacing, and motion for Fluid Island. Use when designing or reviewing UI details, micro-interactions, and design tokens.
license: AGPL-3.0-only
compatibility: opencode
---

## What I do

- Define visual and interaction consistency rules for the island UI.
- Specify spacing, sizing, motion, and state treatments.
- Guide quiet-by-default behavior and glanceability.

## When to use me

Use for UI design/review. For widget implementation, use `gtk4-libadwaita`.

## Design principles

- Quiet by default: no constant motion or attention-seeking.
- Fluid, not flashy: motion communicates state changes.
- Glanceable: information readable at a glance.
- Native: follow GNOME HIG and Adw styling.

## Spacing and sizing (starting points)

- Collapsed pill: ~140x38 px (prototype reference).
- Expanded: ~410x165 px, grows away from clock.
- Corner radius: large (pill shape).
- Padding: 8-12 px internal.
- Gap between clock and pill: 8 px.

## Interaction states

- Collapsed, expanded, hover, focus, disabled, loading.
- Clear focus ring; keyboard activation (Enter/Space).
- Press feedback via subtle scale/opacity, not color-only.

## Motion

- Durations 150-300 ms; easing EASE_OUT_QUAD (prototype).
- Interruptible; no queued animations.
- Reduced-motion: instant or fade-only transitions.
- Avoid layout thrash; animate transform/opacity preferentially.

## Color and contrast

- Use Adw named colors and CSS variables.
- Ensure WCAG AA contrast for text.
- Support high-contrast themes.

## Common mistakes

- Animating width/height causing reflow.
- Relying on color alone for state.
- Ignoring reduced-motion.
- Overly bright/animated idle states.

## Authoritative documentation

- GNOME HIG: https://developer.gnome.org/hig/
- Adw styling: https://gnome.pages.gitlab.gnome.org/libadwaita/doc/
- WCAG: https://www.w3.org/WAI/standards-guidelines/wcag/
