//! Fallback window shown when no GNOME extension is attached.

use crate::state::IslandState;
use crate::stopwatch::{format_duration_cs, Stopwatch};
use crate::system_info;
use gtk4::gdk;
use gtk4::glib::{self as glib, Propagation, SourceId};
use gtk4::prelude::*;
use libadwaita as adw;
use libadwaita::prelude::AdwApplicationWindowExt;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

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

.fluid-island-time {
    font-weight: 700;
    font-size: 1.2em;
}

.fluid-island-detail {
    opacity: 0.72;
    font-size: 0.85em;
}

.fluid-island-lap {
    background-color: rgba(128, 128, 128, 0.2);
    border: 1px solid @borders;
    border-radius: 12px;
    padding: 4px 10px;
}
"#;

fn update_system_info(
    time_label: &gtk4::Label,
    tz_label: &gtk4::Label,
    uptime_label: &gtk4::Label,
    memory_label: &gtk4::Label,
    load_label: &gtk4::Label,
) {
    let (time, tz) = system_info::current_time();
    time_label.set_text(&time);
    tz_label.set_text(tz.as_deref().unwrap_or(""));
    if let Some(uptime) = system_info::read_uptime() {
        uptime_label.set_text(&format!("Uptime: {}", uptime));
    }
    if let Some(memory) = system_info::read_memory_usage() {
        memory_label.set_text(&format!("Memory: {}", memory));
    }
    if let Some(load) = system_info::read_loadavg() {
        load_label.set_text(&format!("Load avg: {}", load));
    }
}

pub struct FallbackWindow {
    window: adw::ApplicationWindow,
    state: Arc<Mutex<IslandState>>,
    toggle: gtk4::Button,
    revealer: gtk4::Revealer,
    timer_id: Rc<RefCell<Option<SourceId>>>,
    stopwatch_timer_id: Rc<RefCell<Option<SourceId>>>,
}

#[derive(Clone)]
struct SysLabels {
    time: gtk4::Label,
    tz: gtk4::Label,
    uptime: gtk4::Label,
    memory: gtk4::Label,
    load: gtk4::Label,
}

impl SysLabels {
    fn update_all(&self) {
        update_system_info(&self.time, &self.tz, &self.uptime, &self.memory, &self.load);
    }

    fn start_polling(&self, slot: &Rc<RefCell<Option<SourceId>>>) {
        if slot.borrow().is_some() {
            return;
        }
        let labels = self.clone();
        let id = glib::timeout_add_local(Duration::from_secs(1), move || {
            labels.update_all();
            glib::ControlFlow::Continue
        });
        *slot.borrow_mut() = Some(id);
    }
}

fn stop_polling(slot: &Rc<RefCell<Option<SourceId>>>) {
    if let Some(id) = slot.borrow_mut().take() {
        id.remove();
    }
}

impl FallbackWindow {
    /// Create and wire the fallback window. The window is not presented by this
    /// constructor; call [`FallbackWindow::present`].
    pub fn new(
        application: &adw::Application,
        state: Arc<Mutex<IslandState>>,
        stopwatch: Arc<Mutex<Stopwatch>>,
    ) -> Self {
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
        let animations_enabled = gtk4::Settings::default()
            .map(|settings| settings.is_gtk_enable_animations())
            .unwrap_or(true);
        revealer.set_transition_duration(if animations_enabled { 180 } else { 0 });
        revealer.set_transition_type(gtk4::RevealerTransitionType::SlideDown);

        let time_label = gtk4::Label::new(None);
        time_label.add_css_class("fluid-island-time");
        time_label.set_xalign(0.0);

        let tz_label = gtk4::Label::new(None);
        tz_label.add_css_class("fluid-island-detail");
        tz_label.set_xalign(0.0);

        let uptime_label = gtk4::Label::new(None);
        uptime_label.add_css_class("fluid-island-detail");
        uptime_label.set_xalign(0.0);

        let memory_label = gtk4::Label::new(None);
        memory_label.add_css_class("fluid-island-detail");
        memory_label.set_xalign(0.0);

        let load_label = gtk4::Label::new(None);
        load_label.add_css_class("fluid-island-detail");
        load_label.set_xalign(0.0);

        let sw_title = gtk4::Label::new(Some("Stopwatch"));
        sw_title.add_css_class("fluid-island-detail");
        sw_title.set_xalign(0.0);

        let stopwatch_label = gtk4::Label::new(Some("00:00,00"));
        stopwatch_label.add_css_class("fluid-island-time");
        stopwatch_label.set_xalign(0.0);
        stopwatch_label.set_hexpand(true);

        let primary_btn = gtk4::Button::with_label("Start");
        primary_btn.set_tooltip_text(Some("Start, pause, or resume the stopwatch"));

        let sw_head_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        sw_head_row.append(&stopwatch_label);
        sw_head_row.append(&primary_btn);

        let chips_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);

        let laps_scroll = gtk4::ScrolledWindow::new();
        laps_scroll.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Never);
        laps_scroll.set_hexpand(true);
        laps_scroll.set_child(Some(&chips_box));

        let lap_btn = gtk4::Button::with_label("Lap");
        lap_btn.set_tooltip_text(Some("Record a lap time"));
        let reset_btn = gtk4::Button::with_label("Reset");
        reset_btn.set_tooltip_text(Some("Reset the stopwatch"));

        let laps_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        laps_row.append(&laps_scroll);
        laps_row.append(&lap_btn);
        laps_row.append(&reset_btn);

        // Collapsed mini-display: live time shown only while the stopwatch runs.
        let sw_mini_label = gtk4::Label::new(Some(""));
        sw_mini_label.add_css_class("fluid-island-time");
        sw_mini_label.set_xalign(0.0);
        sw_mini_label.set_visible(false);

        let detail_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        detail_box.append(&time_label);
        detail_box.append(&tz_label);
        detail_box.append(&uptime_label);
        detail_box.append(&memory_label);
        detail_box.append(&load_label);
        detail_box.append(&sw_title);
        detail_box.append(&sw_head_row);
        detail_box.append(&laps_row);

        revealer.set_child(Some(&detail_box));

        let stopwatch_timer_id = Rc::new(RefCell::new(None));
        let ui_stopwatch = stopwatch.clone();
        {
            let label = stopwatch_label.clone();
            let mini = sw_mini_label.clone();
            let primary = primary_btn.clone();
            let lap = lap_btn.clone();
            let sw = ui_stopwatch.clone();
            let timer_id_ref = stopwatch_timer_id.clone();
            let id = glib::timeout_add_local(Duration::from_millis(100), move || {
                let guard = sw.lock().unwrap_or_else(|p| p.into_inner());
                let elapsed = guard.elapsed();
                let running = guard.is_running();
                drop(guard);
                let text = format_duration_cs(elapsed);
                label.set_text(&text);
                mini.set_text(&text);
                mini.set_visible(running);
                primary.set_label(if running {
                    "Pause"
                } else if elapsed.is_zero() {
                    "Start"
                } else {
                    "Resume"
                });
                lap.set_sensitive(!elapsed.is_zero());
                glib::ControlFlow::Continue
            });
            *timer_id_ref.borrow_mut() = Some(id);
        }

        let ui_stopwatch_for_primary = stopwatch.clone();
        primary_btn.connect_clicked(move |_| {
            let mut sw = ui_stopwatch_for_primary
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if sw.is_running() {
                sw.pause();
            } else if sw.elapsed().is_zero() {
                sw.start();
            } else {
                sw.resume();
            }
        });

        let ui_stopwatch_for_lap = stopwatch.clone();
        let chips_box_for_lap = chips_box.clone();
        let laps_scroll_for_lap = laps_scroll.clone();
        lap_btn.connect_clicked(move |_| {
            let mut sw = ui_stopwatch_for_lap
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Some(lap) = sw.lap() {
                let chip = gtk4::Label::new(Some(&format_duration_cs(lap)));
                chip.add_css_class("fluid-island-lap");
                chips_box_for_lap.append(&chip);
                let adj = laps_scroll_for_lap.hadjustment();
                glib::idle_add_local_once(move || {
                    adj.set_value(adj.upper() - adj.page_size());
                });
            }
        });

        let ui_stopwatch_for_reset = stopwatch.clone();
        let chips_box_for_reset = chips_box.clone();
        reset_btn.connect_clicked(move |_| {
            ui_stopwatch_for_reset
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .reset();
            while let Some(child) = chips_box_for_reset.first_child() {
                chips_box_for_reset.remove(&child);
            }
        });

        island.append(&title);
        island.append(&toggle);
        island.append(&sw_mini_label);
        island.append(&revealer);

        let root = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        root.set_halign(gtk4::Align::Center);
        root.set_valign(gtk4::Align::Center);
        root.append(&island);

        window.set_content(Some(&root));

        let state_for_click = state.clone();
        let state_for_key = state.clone();
        let toggle_for_click = toggle.clone();
        let revealer_for_click = revealer.clone();
        let toggle_for_key = toggle.clone();
        let revealer_for_key = revealer.clone();

        let timer_id = Rc::new(RefCell::new(None));

        let labels = SysLabels {
            time: time_label,
            tz: tz_label,
            uptime: uptime_label,
            memory: memory_label,
            load: load_label,
        };

        let timer_id_for_toggle = timer_id.clone();
        let labels_for_toggle = labels.clone();
        toggle.connect_clicked(move |_| {
            let new_state = {
                let mut guard = state_for_click
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                guard.toggle();
                *guard
            };
            apply_state(new_state, &toggle_for_click, &revealer_for_click);

            if new_state == IslandState::Expanded {
                labels_for_toggle.start_polling(&timer_id_for_toggle);
            } else {
                stop_polling(&timer_id_for_toggle);
            }
        });

        let timer_id_for_key = timer_id.clone();
        let controller = gtk4::EventControllerKey::new();
        controller.connect_key_pressed(move |_controller, key, _keycode, _modifiers| {
            if key == gdk::Key::Escape
                && *state_for_key.lock().unwrap_or_else(|p| p.into_inner()) == IslandState::Expanded
            {
                *state_for_key.lock().unwrap_or_else(|p| p.into_inner()) = IslandState::Collapsed;
                apply_state(IslandState::Collapsed, &toggle_for_key, &revealer_for_key);
                stop_polling(&timer_id_for_key);

                Propagation::Stop
            } else {
                Propagation::Proceed
            }
        });
        window.add_controller(controller);

        apply_state(initial_state, &toggle, &revealer);

        labels.update_all();
        if initial_state == IslandState::Expanded {
            labels.start_polling(&timer_id);
        }

        Self {
            window,
            state,
            toggle,
            revealer,
            timer_id,
            stopwatch_timer_id,
        }
    }

    pub fn present(&self) {
        self.window.present();
    }

    pub fn hide(&self) {
        self.window.set_visible(false);
    }

    pub fn set_state(&self, state: IslandState) {
        let mut guard = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        *guard = state;
        apply_state(state, &self.toggle, &self.revealer);
    }
}

impl Drop for FallbackWindow {
    fn drop(&mut self) {
        if let Some(id) = self.timer_id.borrow_mut().take() {
            id.remove();
        }
        if let Some(id) = self.stopwatch_timer_id.borrow_mut().take() {
            id.remove();
        }
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
