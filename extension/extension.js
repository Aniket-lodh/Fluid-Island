// Fluid Island — GNOME Shell extension (prototype, thin).
// Renders the island pill in the top bar and mirrors core state over D-Bus.
// Holds no business logic; the Rust core (org.fluidisland.Core) is authoritative.
import Clutter from 'gi://Clutter';
import Gio from 'gi://Gio';
import GLib from 'gi://GLib';
import St from 'gi://St';
import * as Main from 'resource:///org/gnome/shell/ui/main.js';
import {Extension} from 'resource:///org/gnome/shell/extensions/extension.js';

const Cairo = imports.cairo;

const BUS_NAME = 'org.fluidisland.Core';
const OBJECT_PATH = '/org/fluidisland/Core';
const IFACE = 'org.fluidisland.Core';
const COLLAPSED = {w: 126, h: 37}; // Mac Dynamic Island idle: 126×37pt (iPhone 14 Pro)
const EXPANDED = {w: 320, h: 180};
const CALL_TIMEOUT_MS = 500;
const GAP = 8;
const TOP_GAP = 6; // float off the top edge — not stuck

export default class FluidIslandExtension extends Extension {
    enable() {
        this._expanded = false;
        this._online = false;
        this._toggleGen = 0;
        this._pendingGen = 0;
        this._sysTimerId = null;
        this._swTimerId = null;
        // First state sync must not animate: panel geometry may not be
        // settled yet, which otherwise shows the pill flying in from below.
        this._initialSync = true;

        // The island body is a Cairo-drawn superellipse (true continuous
        // curvature), not a CSS border-radius rectangle. It repaints on every
        // allocation change so the morph animation stays crisp mid-flight.
        this._actor = new St.DrawingArea({
            name: 'fluid-island',
            reactive: true,
            style: 'margin-top: 4px;',
            x_expand: false,
        });
        this._actor.connect('repaint', () => this._drawIsland());
        this._actor.connect('notify::allocation', () => this._actor.queue_repaint());

        // DISABLED UI (kept for the upcoming UI pass): pill text, detail panel,
        // system-info labels, stopwatch display, controls, and laps row.
        // The core (org.fluidisland.Core) still owns all of this state and data.
        // this._mainBox = new St.BoxLayout({vertical: true, x_expand: true});
        // this._actor.add_child(this._mainBox);
        //
        // this._label = new St.Label({text: 'island', style: 'color: white; font-weight: bold;'});
        // this._mainBox.add_child(this._label);
        //
        // this._detailBox = new St.BoxLayout({vertical: true, style: 'spacing: 4px; padding-top: 6px;'});
        // this._timeLabel = new St.Label({text: '', style: 'color: white; font-weight: bold; font-size: 1.1em;'});
        // this._tzLabel = new St.Label({text: '', style: 'color: #ccc; font-size: 0.8em;'});
        // this._uptimeLabel = new St.Label({text: '', style: 'color: #ccc; font-size: 0.85em;'});
        // this._memoryLabel = new St.Label({text: '', style: 'color: #ccc; font-size: 0.85em;'});
        // this._loadLabel = new St.Label({text: '', style: 'color: #ccc; font-size: 0.85em;'});
        // this._detailBox.add_child(this._timeLabel);
        // this._detailBox.add_child(this._tzLabel);
        // this._detailBox.add_child(this._uptimeLabel);
        // this._detailBox.add_child(this._memoryLabel);
        // this._detailBox.add_child(this._loadLabel);
        //
        // this._swTitle = new St.Label({text: 'Stopwatch', style: 'color: #ccc; font-size: 0.85em;'});
        // this._detailBox.add_child(this._swTitle);
        // this._swHeadRow = new St.BoxLayout({style: 'spacing: 8px;'});
        // this._stopwatchLabel = new St.Label({text: '00:00,00', style: 'color: white; font-weight: bold; font-size: 1.4em;', x_expand: true});
        // this._swPrimaryBtn = new St.Button({label: 'Start', style_class: 'button'});
        // this._swHeadRow.add_child(this._stopwatchLabel);
        // this._swHeadRow.add_child(this._swPrimaryBtn);
        // this._detailBox.add_child(this._swHeadRow);
        //
        // this._lapsRow = new St.BoxLayout({style: 'spacing: 6px;'});
        // this._lapsScroll = new St.ScrollView({hscrollbar_policy: St.PolicyType.NEVER, vscrollbar_policy: St.PolicyType.NEVER, x_expand: true});
        // this._lapsBox = new St.BoxLayout({style: 'spacing: 6px;'});
        // this._lapsScroll.set_child(this._lapsBox);
        // this._lapBtn = new St.Button({label: 'Lap', style_class: 'button'});
        // this._swResetBtn = new St.Button({label: 'Reset', style_class: 'button'});
        // this._lapsRow.add_child(this._lapsScroll);
        // this._lapsRow.add_child(this._lapBtn);
        // this._lapsRow.add_child(this._swResetBtn);
        // this._detailBox.add_child(this._lapsRow);
        //
        // this._swStatus = 'stopped';
        // this._swPrimaryBtn.connect('clicked', () => {
        //     const method = this._swStatus === 'running' ? 'StopwatchPause'
        //         : this._swStatus === 'paused' ? 'StopwatchResume' : 'StopwatchStart';
        //     this._callStopwatch(method);
        //     this._updateStopwatchSoon();
        // });
        // this._lapBtn.connect('clicked', () => {
        //     this._callStopwatch('StopwatchLap');
        // });
        // this._swResetBtn.connect('clicked', () => {
        //     this._callStopwatch('StopwatchReset');
        //     this._rebuildLaps([]);
        // });
        //
        // this._mainBox.add_child(this._detailBox);
        // this._detailBox.visible = false;

        Main.layoutManager.addTopChrome(this._actor, {affectsStruts: false, trackFullscreen: true});
        this._place(COLLAPSED, false);

        // Top-bar detection: if panel is at top, we become the center notch
        this._isTop = this._isTopBar();
        this._centerBox = Main.panel._centerBox;
        this._panelBox = Main.layoutManager.panelBox;
        this._dateMenuContainer = Main.panel.statusArea.dateMenu.container;
        this._panelBoxId = this._panelBox.connect('notify::y', () => this._onPanelPositionChanged());

        if (this._isTop) {
            this._hideCenterClock();
        }

        // Re-check after allocation — at shell startup panelBox y is NaN, then becomes 0
        GLib.timeout_add(GLib.PRIORITY_DEFAULT, 600, () => {
            if (!this._actor) return GLib.SOURCE_REMOVE;
            const nowTop = this._isTopBar();
            if (nowTop !== this._isTop) {
                this._onPanelPositionChanged();
            } else {
                this._place(this._size(), false);
            }
            return GLib.SOURCE_REMOVE;
        });

        this._clickId = this._actor.connect('button-press-event', () => {
            this._onClick();
            return Clutter.EVENT_STOP;
        });
        this._monitorsId = Main.layoutManager.connect('monitors-changed', () => this._place(this._size(), false));
        const clock = Main.panel.statusArea.dateMenu.container;
        this._clockIds = [
            clock.connect('notify::width', () => { if (!this._isTop) this._replace(); }),
            clock.connect('notify::x', () => { if (!this._isTop) this._replace(); }),
        ];
        this._clock = clock;

        this._watchId = Gio.bus_watch_name(Gio.BusType.SESSION, BUS_NAME, Gio.BusNameWatcherFlags.NONE,
            () => this._onAppAppeared(), () => this._onAppVanished());

        this._signalId = Gio.DBus.session.signal_subscribe(BUS_NAME, IFACE, 'StateChanged', OBJECT_PATH,
            null, Gio.DBusSignalFlags.NONE,
            (_c, _s, _p, _i, _sig, params) => {
                const [state] = params.deep_unpack();
                this._pendingGen = this._toggleGen;
                this._applyState(state === 'expanded', true);
            });

        // Ensure core is running instantly (like other pure-JS extensions).
        // If the D-Bus name is not owned, spawn the headless core directly
        // so we don't wait for systemd's graphical-session startup.
        this._ensureCoreRunning();

        console.log('fluid-island: enabled');
    }

    _ensureCoreRunning() {
        console.log('fluid-island: ensuring core running');
        Gio.DBus.session.call(BUS_NAME, OBJECT_PATH, IFACE, 'GetState', null,
            GLib.VariantType.new('(s)'), Gio.DBusCallFlags.NONE, 200, null,
            (conn, res) => {
                try {
                    conn.call_finish(res);
                    console.log('fluid-island: core already running');
                } catch (e) {
                    console.log(`fluid-island: GetState failed: ${e.message}`);
                    if (!e.message.includes('ServiceUnknown') && !e.message.includes('NameHasNoOwner')) return;
                    console.log('fluid-island: core not running, spawning');
                    const candidates = [
                        '/media/debian/Data/Aniket-Property/Things/opensource_projects/fluid-island/target/release/fluid-island',
                        GLib.find_program_in_path('fluid-island'),
                    ].filter(Boolean);
                    for (const path of candidates) {
                        try {
                            const proc = Gio.Subprocess.new([path], Gio.SubprocessFlags.NONE);
                            console.log(`fluid-island: spawned core at ${path}`);
                            return;
                        } catch (err) {
                            console.log(`fluid-island: Gio.Subprocess spawn failed at ${path}: ${err.message}, trying GLib.spawn`);
                            try {
                                GLib.spawn_async(null, [path], null, GLib.SpawnFlags.SEARCH_PATH, null);
                                console.log(`fluid-island: spawned core via GLib.spawn_async at ${path}`);
                                return;
                            } catch (err2) {
                                console.log(`fluid-island: GLib.spawn failed at ${path}: ${err2.message}`);
                            }
                        }
                    }
                }
            });
    }

    // DISABLED: stopwatch D-Bus calls re-enabled with the UI pass.
    // _callStopwatch(method) {
    //     Gio.DBus.session.call(BUS_NAME, OBJECT_PATH, IFACE, method, null, null, Gio.DBusCallFlags.NONE, CALL_TIMEOUT_MS, null, () => {});
    // }

    // DISABLED: system-info polling re-enabled with the UI pass.
    // _updateSystemInfo() {
    //     Gio.DBus.session.call(BUS_NAME, OBJECT_PATH, IFACE, 'GetSystemInfo', null,
    //         GLib.VariantType.new('(sssss)'), Gio.DBusCallFlags.NONE, CALL_TIMEOUT_MS, null,
    //         (conn, res) => {
    //             try {
    //                 const [time, tz, uptime, memory, load] = conn.call_finish(res).deep_unpack();
    //                 this._timeLabel.text = time;
    //                 this._tzLabel.text = tz;
    //                 this._uptimeLabel.text = uptime ? `Uptime: ${uptime}` : '';
    //                 this._memoryLabel.text = memory ? `Memory: ${memory}` : '';
    //                 this._loadLabel.text = load ? `Load avg: ${load}` : '';
    //             } catch (e) {
    //             }
    //         });
    // }

    // DISABLED: stopwatch polling re-enabled with the UI pass.
    // _updateStopwatch() {
    //     Gio.DBus.session.call(BUS_NAME, OBJECT_PATH, IFACE, 'GetStopwatch', null,
    //         GLib.VariantType.new('(ss)'), Gio.DBusCallFlags.NONE, CALL_TIMEOUT_MS, null,
    //         (conn, res) => {
    //             try {
    //                 const [elapsed, status] = conn.call_finish(res).deep_unpack();
    //                 this._stopwatchLabel.text = elapsed;
    //                 this._swStatus = status;
    //                 this._swPrimaryBtn.label = status === 'running' ? 'Pause'
    //                     : status === 'paused' ? 'Resume' : 'Start';
    //                 if (!this._expanded) {
    //                     if (status === 'running') {
    //                         this._label.text = elapsed;
    //                         this._label.style = 'color: #ff9f0a; font-weight: bold;';
    //                     } else {
    //                         this._label.text = 'island';
    //                         this._label.style = 'color: white; font-weight: bold;';
    //                     }
    //                 }
    //             } catch (e) {}
    //         });
    // }

    // DISABLED: re-enabled with the UI pass.
    // _updateStopwatchSoon() {
    //     GLib.timeout_add(GLib.PRIORITY_DEFAULT, 120, () => {
    //         this._updateStopwatch();
    //         this._updateLaps();
    //         return GLib.SOURCE_REMOVE;
    //     });
    // }

    // DISABLED: laps UI re-enabled with the UI pass.
    // _updateLaps() {
    //     Gio.DBus.session.call(BUS_NAME, OBJECT_PATH, IFACE, 'GetLaps', null,
    //         GLib.VariantType.new('(as)'), Gio.DBusCallFlags.NONE, CALL_TIMEOUT_MS, null,
    //         (conn, res) => {
    //             try {
    //                 const [laps] = conn.call_finish(res).deep_unpack();
    //                 this._rebuildLaps(laps);
    //             } catch (e) {}
    //         });
    // }

    // DISABLED: laps UI re-enabled with the UI pass.
    // _rebuildLaps(laps) {
    //     const children = this._lapsBox.get_children().slice();
    //     for (const c of children) {
    //         this._lapsBox.remove_child(c);
    //         c.destroy();
    //     }
    //     for (const lap of laps) {
    //         const chip = new St.Label({text: lap, style: 'color: white; background-color: rgba(128,128,128,0.25); border-radius: 12px; padding: 4px 10px;'});
    //         this._lapsBox.add_child(chip);
    //     }
    //     const adj = this._lapsScroll.hadjustment;
    //     if (adj) adj.set_value(Math.max(0, adj.upper - adj.page_size));
    // }

    // DISABLED: polling re-enabled with the UI pass.
    // _startPolling() {
    //     this._stopPolling();
    //     this._updateSystemInfo();
    //     this._updateStopwatch();
    //     this._updateLaps();
    //     this._sysTimerId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 1000, () => {
    //         this._updateSystemInfo();
    //         this._updateLaps();
    //         return GLib.SOURCE_CONTINUE;
    //     });
    //     this._swTimerId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
    //         this._updateStopwatch();
    //         return GLib.SOURCE_CONTINUE;
    //     });
    // }

    // DISABLED: polling re-enabled with the UI pass.
    // _startSwPolling() {
    //     if (this._swTimerId) return;
    //     this._updateStopwatch();
    //     this._swTimerId = GLib.timeout_add(GLib.PRIORITY_DEFAULT, 100, () => {
    //         this._updateStopwatch();
    //         return GLib.SOURCE_CONTINUE;
    //     });
    // }

    // DISABLED: polling re-enabled with the UI pass.
    // _stopSwPolling() {
    //     if (this._swTimerId) {
    //         GLib.source_remove(this._swTimerId);
    //         this._swTimerId = null;
    //     }
    //     if (!this._expanded) {
    //         this._label.text = 'island';
    //         this._label.style = 'color: white; font-weight: bold;';
    //     }
    // }

    // DISABLED: polling re-enabled with the UI pass.
    // _stopPolling() {
    //     if (this._sysTimerId) {
    //         GLib.source_remove(this._sysTimerId);
    //         this._sysTimerId = null;
    //     }
    //     if (this._swTimerId) {
    //         GLib.source_remove(this._swTimerId);
    //         this._swTimerId = null;
    //     }
    // }

    _size() {
        return this._expanded ? EXPANDED : COLLAPSED;
    }

    _replace() {
        if (this._actor)
            this._place(this._size(), false);
    }

    _onClick() {
        if (!this._online) {
            console.log('fluid-island: click while offline, reconciling via GetState');
            this._reconcile();
            return;
        }
        const gen = ++this._toggleGen;
        this._pendingGen = gen;
        Gio.DBus.session.call(BUS_NAME, OBJECT_PATH, IFACE, 'Toggle', null,
            GLib.VariantType.new('(s)'), Gio.DBusCallFlags.NONE, CALL_TIMEOUT_MS, null,
            (conn, res) => {
                try {
                    const [state] = conn.call_finish(res).deep_unpack();
                    if (gen !== this._pendingGen && gen < this._pendingGen) {
                        console.log(`fluid-island: ignoring stale Toggle reply gen=${gen} pending=${this._pendingGen}`);
                        return;
                    }
                    this._pendingGen = gen;
                    this._applyState(state === 'expanded', true);
                } catch (e) {
                    console.log(`fluid-island: Toggle failed: ${e.message}`);
                    if (e.message.includes('ServiceUnknown') || e.message.includes('NameHasNoOwner')) {
                        this._online = false;
                    }
                    this._reconcile();
                }
            });
    }

    _reconcile(attempt = 0) {
        const gen = this._pendingGen;
        Gio.DBus.session.call(BUS_NAME, OBJECT_PATH, IFACE, 'GetState', null,
            GLib.VariantType.new('(s)'), Gio.DBusCallFlags.NONE, CALL_TIMEOUT_MS, null,
            (conn, res) => {
                try {
                    const [state] = conn.call_finish(res).deep_unpack();
                    if (gen !== this._pendingGen && this._pendingGen > gen) {
                        return;
                    }
                    this._applyState(state === 'expanded', true);
                    this._online = true;
                } catch (e) {
                    console.log(`fluid-island: GetState reconcile failed (attempt ${attempt}): ${e.message}`);
                    if (attempt < 3 && (e.message.includes('ServiceUnknown') || e.message.includes('NameHasNoOwner'))) {
                        GLib.timeout_add(GLib.PRIORITY_DEFAULT, 500, () => {
                            this._reconcile(attempt + 1);
                            return GLib.SOURCE_REMOVE;
                        });
                    }
                }
            });
    }

    _onAppAppeared() {
        this._online = true;
        console.log('fluid-island: core appeared, calling Attach');
        this._attachWithRetry(0);
    }

    _attachWithRetry(attempt) {
        const gen = ++this._toggleGen;
        this._pendingGen = gen;
        Gio.DBus.session.call(BUS_NAME, OBJECT_PATH, IFACE, 'Attach', null,
            GLib.VariantType.new('(s)'), Gio.DBusCallFlags.NONE, CALL_TIMEOUT_MS, null,
            (conn, res) => {
                try {
                    const [state] = conn.call_finish(res).deep_unpack();
                    if (gen !== this._pendingGen && gen < this._pendingGen) return;
                    this._pendingGen = gen;
                    this._applyState(state === 'expanded', true);
                } catch (e) {
                    console.log(`fluid-island: Attach failed (attempt ${attempt}): ${e.message}`);
                    if (attempt < 5) {
                        GLib.timeout_add(GLib.PRIORITY_DEFAULT, 500, () => {
                            this._attachWithRetry(attempt + 1);
                            return GLib.SOURCE_REMOVE;
                        });
                    } else {
                        this._reconcile();
                    }
                }
            });
    }

    _onAppVanished() {
        this._online = false;
        console.log('fluid-island: core vanished, resetting to collapsed');
        this._pendingGen = ++this._toggleGen;
        this._applyState(false, true);
    }

    _roundedRectPoint(w, h, r, theta) {
        const a = w / 2, b = h / 2;
        const hx = Math.max(a - r, 0), hy = Math.max(b - r, 0);
        const ux = Math.cos(theta), uy = Math.sin(theta);
        const dist = (t) => {
            const px = Math.abs(t * ux) - hx, py = Math.abs(t * uy) - hy;
            return Math.hypot(Math.max(px, 0), Math.max(py, 0));
        };
        let lo = 0, hi = Math.hypot(a, b) + r;
        for (let i = 0; i < 24; i++) {
            const mid = (lo + hi) / 2;
            if (dist(mid) < r) lo = mid;
            else hi = mid;
        }
        const t = (lo + hi) / 2;
        return [a + t * ux, b + t * uy];
    }

    _drawIsland() {
        const w = this._actor.width, h = this._actor.height;
        if (w <= 0 || h <= 0) return;
        // Single path: rounded-rect and superellipse sampled with the same
        // angular parameter, interpolated by live width. Collapsed is exactly
        // the old r=18 rect, expanded exactly the n=6 squircle, continuous
        // morph between them, no pops.
        const t = Math.min(1, Math.max(0, (w - COLLAPSED.w) / (EXPANDED.w - COLLAPSED.w)));
        const n = 6, steps = 96, r = 18;
        const a = w / 2, b = h / 2;
        const cr = this._actor.get_context();
        cr.setOperator(Cairo.Operator.CLEAR);
        cr.paint();
        cr.setOperator(Cairo.Operator.OVER);
        cr.setSourceRGBA(0, 0, 0, 1);
        for (let i = 0; i <= steps; i++) {
            const th = (i / steps) * Math.PI * 2;
            const c = Math.cos(th), s = Math.sin(th);
            const sx = a + a * Math.sign(c) * Math.pow(Math.abs(c), 2 / n);
            const sy = b + b * Math.sign(s) * Math.pow(Math.abs(s), 2 / n);
            const [rx, ry] = this._roundedRectPoint(w, h, r, th);
            const x = rx + (sx - rx) * t;
            const y = ry + (sy - ry) * t;
            if (i === 0) cr.moveTo(x, y);
            else cr.lineTo(x, y);
        }
        cr.closePath();
        cr.fill();
    }

    _applyState(expanded, animate) {
        this._expanded = expanded;
        // Respect reduced-motion: SHELL's St.Settings enable-animations (verified via Shell source)
        const enableAnims = St.Settings.get().enable_animations;
        const doAnim = animate && enableAnims && !this._initialSync;
        this._initialSync = false;
        this._place(this._size(), doAnim);
        // UI content re-enabled with the UI pass; state sync only for now.
    }

    _isTopBar() {
        const m = Main.layoutManager.primaryMonitor;
        if (!m) return true;
        const [, y] = Main.layoutManager.panelBox.get_transformed_position();
        // At shell startup y is NaN before allocation — default to top (the common case)
        if (Number.isNaN(y)) {
            console.log(`fluid-island: panel y=NaN (not yet allocated) defaulting to isTop=true`);
            return true;
        }
        console.log(`fluid-island: panel y=${y} monitor h=${m.height} isTop=${y < m.height / 2}`);
        return y < m.height / 2;
    }

    _onPanelPositionChanged() {
        const wasTop = this._isTop;
        this._isTop = this._isTopBar();
        if (wasTop === this._isTop) return;
        if (this._isTop) {
            this._hideCenterClock();
        } else {
            this._showCenterClock();
        }
        this._place(this._size(), false);
    }

    _hideCenterClock() {
        // Move the original dateMenu (with its calendar menu) to the right side
        // to preserve its functionality, instead of creating a fake label.
        if (this._dateMenuContainer.get_parent() !== Main.panel._rightBox) {
            this._centerBox.remove_child(this._dateMenuContainer);
            Main.panel._rightBox.insert_child_at_index(this._dateMenuContainer, 0);
            this._dateMenuContainer.show();
        }
        this._centerBox.hide();
        console.log('fluid-island: hid center clock for notch, moved dateMenu to right');
    }

    _showCenterClock() {
        if (this._dateMenuContainer.get_parent() === Main.panel._rightBox) {
            Main.panel._rightBox.remove_child(this._dateMenuContainer);
            this._centerBox.add_child(this._dateMenuContainer);
        }
        this._centerBox.show();
        this._dateMenuContainer.show();
        console.log('fluid-island: restored center clock');
    }

    _clockRect() {
        const clock = Main.panel.statusArea.dateMenu.container;
        const [x, y] = clock.get_transformed_position();
        return {x, y, w: clock.width, h: clock.height};
    }

    _targetPosition(size) {
        const m = Main.layoutManager.primaryMonitor;
        const left = m.x, right = m.x + m.width;
        const panelY = this._isTop ? m.y + TOP_GAP : m.y + m.height - Main.panel.height;
        if (this._isTop) {
            // Center notch: pill is centered, grows symmetrically but clamped
            const centerX = left + m.width / 2;
            const w = Math.min(size.w, right - left - 2 * GAP);
            const x = Math.round(centerX - w / 2);
            return {x: Math.max(left, Math.min(x, right - w)), y: panelY, w: Math.round(w), h: size.h};
        }
        // Bottom or unknown: right-of-clock (no notch conflict)
        const c = this._clockRect();
        const pillX = c.x + c.w + GAP;
        const w = Math.min(size.w, right - pillX - GAP);
        const x = pillX;
        return {x: Math.round(x), y: panelY, w: Math.round(w), h: size.h};
    }

    _place(size, animate) {
        const {x, y, w, h} = this._targetPosition(size);
        if (!animate) {
            this._actor.set_position(x, y);
            this._actor.set_size(w, h);
            return;
        }
        // Verified Clutter.AnimationMode values exist in libmutter-clutter-16.so (strings dump):
        // EASE_OUT_EXPO and EASE_OUT_BACK are present; EASE_OUT_QUAD was already used successfully.
        const isExpand = size.w === EXPANDED.w;
        this._actor.ease({
            x, y, width: w, height: h,
            duration: isExpand ? 420 : 280,
            mode: isExpand ? Clutter.AnimationMode.EASE_OUT_EXPO : Clutter.AnimationMode.EASE_IN_OUT_CUBIC,
        });
        // Subtle scale overshoot on expand for liquid feel (animatable: scale_x/y)
        if (isExpand) {
            this._actor.set_scale(0.96, 0.96);
            this._actor.ease({
                scale_x: 1.0, scale_y: 1.0,
                duration: 450,
                mode: Clutter.AnimationMode.EASE_OUT_BACK,
            });
        }
    }

    disable() {
        // Polling timers re-enabled with the UI pass; guard in case any exist.
        // (Timer init lines kept commented below for the UI pass.)
        // if (this._sysTimerId) { GLib.source_remove(this._sysTimerId); this._sysTimerId = null; }
        // if (this._swTimerId) { GLib.source_remove(this._swTimerId); this._swTimerId = null; }
        if (this._panelBoxId) {
            Main.layoutManager.panelBox.disconnect(this._panelBoxId);
            this._panelBoxId = null;
        }
        // Restore center clock if we moved it to the right
        if (this._isTop) {
            this._showCenterClock();
        }
        if (this._online) {
            Gio.DBus.session.call(BUS_NAME, OBJECT_PATH, IFACE, 'Detach', null, null,
                Gio.DBusCallFlags.NONE, CALL_TIMEOUT_MS, null, () => {});
        }
        if (this._signalId) {
            Gio.DBus.session.signal_unsubscribe(this._signalId);
            this._signalId = null;
        }
        if (this._watchId) {
            Gio.bus_unwatch_name(this._watchId);
            this._watchId = null;
        }
        if (this._monitorsId) {
            Main.layoutManager.disconnect(this._monitorsId);
            this._monitorsId = null;
        }
        if (this._clockIds) {
            this._clockIds.forEach(id => this._clock.disconnect(id));
            this._clockIds = null;
        }
        if (this._clickId) {
            this._actor.disconnect(this._clickId);
            this._clickId = null;
        }
        Main.layoutManager.removeChrome(this._actor);
        this._actor.destroy();
        this._actor = null;
        console.log('fluid-island: disabled and cleaned up');
    }
}
