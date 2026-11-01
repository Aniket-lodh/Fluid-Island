//! Fallback window shown when no GNOME extension is attached.

use crate::state::IslandState;
use gtk4::gdk;
use gtk4::glib::Propagation;
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::AdwApplicationWindowExt;
use std::sync::{Arc, Mutex};

const ISLAND_CSS: &str = r#"
.fluid-island {
    background-color: @window_bg_color;
    border: 1px solid @borders;
    border-radius: 24px;
    padding: 14px 18px;
}

.fluid-island-title {
    font-weight: 700;
}

.fluid-island-detail {
    opacity: 0.72;
}
"#;

/// A compact, island-inspired window exposing the island state.
#[derive(Clone)]
pub struct FallbackWindow {
    window: adw::ApplicationWindow,
    state: Arc<Mutex<IslandState>>,
    toggle: gtk4::Button,
    revealer: gtk4::Revealer,
}

impl FallbackWindow {
    /// Create and wire the fallback window. The window is not presented by this
    /// constructor; call [`FallbackWindow::present`].
    pub fn new(application: &adw::Application, state: Arc<Mutex<IslandState>>) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(application)
            .title("Fluid Island")
            .decorated(false)
            .resizable(false)
            .default_width(240)
            .build();

        let provider = gtk4::CssProvider::new();
        provider.load_from_string(ISLAND_CSS);
        gtk4::style_context_add_provider_for_display(
            &gtk4::prelude::WidgetExt::display(&window),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let island = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        island.add_css_class("fluid-island");
        island.set_halign(gtk4::Align::Center);
        island.set_valign(gtk4::Align::Center);

        let title = gtk4::Label::new(Some("Fluid Island"));
        title.add_css_class("fluid-island-title");

        let initial_state = state.lock().map(|guard| *guard).unwrap_or_default();
        let toggle = gtk4::Button::with_label(button_label(initial_state));
        toggle.set_tooltip_text(Some("Toggle the island state"));

        let revealer = gtk4::Revealer::new();
        // GTK's animation preference is the available reduced-motion signal in
        // this crate version. If settings are unavailable, keep animation enabled.
        let animations_enabled = gtk4::Settings::default()
            .map(|settings| settings.is_gtk_enable_animations())
            .unwrap_or(true);
        revealer.set_transition_duration(if animations_enabled { 180 } else { 0 });
        revealer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);

        let detail = gtk4::Label::new(Some("Expanded content area"));
        detail.add_css_class("fluid-island-detail");
        let detail_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        detail_box.append(&detail);
        revealer.set_child(Some(&detail_box));

        island.append(&title);
        island.append(&toggle);
        island.append(&revealer);

        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.set_halign(gtk4::Align::Center);
        root.set_valign(gtk4::Align::Center);
        root.append(&island);

        window.set_content(Some(&root));

        let state_for_click = state.clone();
        let toggle_for_click = toggle.clone();
        let revealer_for_click = revealer.clone();
        toggle.connect_clicked(move |_| {
            let new_state = {
                let mut guard = state_for_click
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                guard.toggle();
                *guard
            };
            apply_state(new_state, &toggle_for_click, &revealer_for_click);
        });

        let state_for_key = state.clone();
        let toggle_for_key = toggle.clone();
        let revealer_for_key = revealer.clone();
        let controller = gtk4::EventControllerKey::new();
        controller.connect_key_pressed(move |_controller, key, _keycode, _modifiers| {
            if key == gdk::Key::Escape
                && *state_for_key.lock().unwrap_or_else(|p| p.into_inner()) == IslandState::Expanded
            {
                *state_for_key.lock().unwrap_or_else(|p| p.into_inner()) = IslandState::Collapsed;
                apply_state(IslandState::Collapsed, &toggle_for_key, &revealer_for_key);
                Propagation::Stop
            } else {
                Propagation::Proceed
            }
        });
        window.add_controller(controller);

        apply_state(initial_state, &toggle, &revealer);

        Self {
            window,
            state,
            toggle,
            revealer,
        }
    }

    /// Present the window to the user.
    pub fn present(&self) {
        self.window.present();
    }

    /// Hide the window.
    pub fn hide(&self) {
        self.window.set_visible(false);
    }

    /// Update the shared state and UI to the supplied value.
    pub fn set_state(&self, state: IslandState) {
        if let Ok(mut guard) = self.state.lock() {
            *guard = state;
        }
        apply_state(state, &self.toggle, &self.revealer);
    }
}

fn button_label(state: IslandState) -> &'static str {
    match state {
        IslandState::Collapsed => "Expand",
        IslandState::Expanded => "Collapse",
    }
}

fn apply_state(state: IslandState, toggle: &gtk4::Button, revealer: &gtk4::Revealer) {
    toggle.set_label(button_label(state));
    revealer.set_reveal_child(state == IslandState::Expanded);
}
