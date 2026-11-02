#!/bin/sh
# Fast extension dev loop: runs the pill in an isolated nested GNOME Shell.
# No logout needed, real session untouched. Ctrl+C or close the window to stop.
set -e
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
RT=/tmp/opencode/gnome
BUS=$RT/bus.sock

cleanup() {
    # Idempotent: trap (INT/TERM) and normal exit both land here.
    if [ -n "$CLEANED" ]; then return 0; fi
    CLEANED=1
    echo "--- stopping nested session ---"
    if [ -n "$SHELL_PID" ]; then kill $SHELL_PID 2>/dev/null || true; fi
    pkill -f "gnome-shell --nested" 2>/dev/null || true
    if [ -n "$CORE_PID" ]; then kill $CORE_PID 2>/dev/null || true; fi
    if [ -n "$DAEMON_PID" ]; then kill $DAEMON_PID 2>/dev/null; wait $DAEMON_PID 2>/dev/null || true; fi
    rm -f "$BUS" /tmp/dev-bus-addr
    if [ -n "$WL_DISPLAY" ]; then rm -f "/run/user/1000/$WL_DISPLAY" "/run/user/1000/$WL_DISPLAY.lock"; fi
    echo "--- cleaned up ---"
    exit 0
}
trap cleanup INT TERM

pkill -f "gnome-shell --nested" 2>/dev/null || true
rm -rf "$RT/home" "$RT/config" "$RT/data" "$RT/cache" "$BUS"
mkdir -p "$RT/home" "$RT/config/glib-2.0/settings" "$RT/data/gnome-shell/extensions" "$RT/cache"

cat > "$RT/config/glib-2.0/settings/keyfile" <<'EOF'
[org/gnome/shell]
enabled-extensions=['fluid-island@fluid-island.local']
disable-user-extensions=false
EOF

cp -r "$ROOT/extension" "$RT/data/gnome-shell/extensions/fluid-island@fluid-island.local"

dbus-daemon --session --address=unix:path=$BUS --nofork --print-address > /tmp/dev-bus-addr 2>/dev/null &
DAEMON_PID=$!
sleep 1
export DBUS_SESSION_BUS_ADDRESS=unix:path=$BUS

"$ROOT/target/release/fluid-island" &
CORE_PID=$!
sleep 1

echo "--- nested shell starting (close its window or Ctrl+C here to stop) ---"
WL_DISPLAY="wayland-island-$$"
rm -f "/run/user/1000/$WL_DISPLAY" "/run/user/1000/$WL_DISPLAY.lock"
HOME=$RT/home \
XDG_CONFIG_HOME=$RT/config \
XDG_DATA_HOME=$RT/data \
XDG_CACHE_HOME=$RT/cache \
GSETTINGS_BACKEND=keyfile \
DISPLAY=:1 \
gnome-shell --nested --wayland-display="$WL_DISPLAY" --unsafe-mode &
SHELL_PID=$!

echo "core=$CORE_PID shell=$SHELL_PID bus=$DAEMON_PID"
echo "logs: tail -f /tmp/dev-shell.log | grep fluid-island"
wait $SHELL_PID 2>/dev/null || true
cleanup
