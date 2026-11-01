---
name: gtk4-libadwaita
description: Native GTK4/libadwaita UI for Fluid Island: layouts, accessibility, styling, and animations. Use when building or reviewing widgets, windows, transitions, and visual states.
license: AGPL-3.0-only
compatibility: opencode
---

## What I do

- Guide GTK4/libadwaita widget composition and layout.
- Cover accessibility, theming, and adaptive behavior.
- Define animation and transition practices.
- Highlight GTK4 limitations relevant to Fluid Island (positioning).

## When to use me

Use for UI implementation/review in the Rust core. For extension code, use
`gnome-shell-extension`. For design tokens/spacing/motion specs, use
`desktop-ui-design`.

## Key GTK4 facts

- No client-side toplevel positioning API; compositor places windows.
- Use libadwaita for adaptive layouts, styling, and GNOME HIG alignment.
- Prefer CSS classes and Adw style classes over custom drawing where possible.
- Use `Gtk::Revealer`, transitions, and property animations carefully.

## Layout and widgets

- Build with composite templates or builder XML where it aids clarity.
- Keep view logic thin; push state changes through a single update path.
- Use `Adw::Clamp`, `Adw::ToastOverlay`, `Adw::StatusPage` as appropriate.
- Avoid deep nesting; prefer named containers and clear hierarchy.

## Accessibility

- Set accessible roles, labels, and descriptions (`set_accessible_role`,
  `update_property`).
- Ensure keyboard navigation and focus visibility.
- Honor `prefers-reduced-motion` by shortening/disabling animations.
- Test with high-contrast and large text settings.

## Animations

- Use `Adw::TimedAnimation` / `Adw::SpringAnimation` for property changes.
- Keep durations short (150-300 ms) and easing purposeful.
- Make animations interruptible; cancel on state change.
- Avoid animating layout properties that cause reflow storms.
- Measure frame times; target 60 fps but do not assume vsync.

## Styling

- Use Adw CSS variables and named colors; avoid hardcoded hex where possible.
- Support light/dark automatically via Adw.
- Keep custom CSS minimal and documented.

## Commands

```sh
cargo build
cargo test
# Run with GTK_DEBUG / GSK_DEBUG for rendering diagnostics if needed
GTK_DEBUG=interactive cargo run
```

## Common mistakes

- Assuming `move()` exists for toplevels.
- Blocking the main loop during animation.
- Forgetting to unparent/clean up widgets on teardown.
- Hardcoding colors that break high-contrast.
- Animating without cancellation, causing stuck states.

## Authoritative documentation

- GTK4 docs: https://docs.gtk.org/gtk4/
- libadwaita docs: https://gnome.pages.gitlab.gnome.org/libadwaita/doc/
- gtk-rs book: https://gtk-rs.org/gtk-rs-book/
- GNOME HIG: https://developer.gnome.org/hig/
