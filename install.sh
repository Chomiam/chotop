#!/bin/bash
# Chotop installation script

set -e

echo "=== Chotop Installer ==="
echo "Discord Overlay Daemon for Wayland"
echo ""

# Check dependencies
echo "Checking dependencies..."
MISSING_DEPS=()

if ! pkg-config --exists gtk4; then
    MISSING_DEPS+=("gtk4")
fi

if ! pkg-config --exists gtk4-layer-shell-0; then
    MISSING_DEPS+=("gtk4-layer-shell")
fi

if [ ${#MISSING_DEPS[@]} -ne 0 ]; then
    echo "Missing dependencies: ${MISSING_DEPS[*]}"
    echo ""
    echo "Install with:"
    echo "  Arch: sudo pacman -S gtk4 gtk4-layer-shell"
    echo "  Debian/Ubuntu: sudo apt install libgtk-4-dev libgtk4-layer-shell-dev"
    exit 1
fi

echo "All dependencies found!"
echo ""

# Build
echo "Building Chotop..."
cargo build --release

# Install binary
echo "Installing to ~/.local/bin..."
mkdir -p ~/.local/bin
cp target/release/discord-overlay-daemon ~/.local/bin/
chmod +x ~/.local/bin/discord-overlay-daemon

# Create wrapper for Equibop if not exists
if command -v equibop &> /dev/null; then
    echo "Equibop found! Creating wrapper script..."
    cat > ~/.local/bin/equibop-overlay << 'EOF'
#!/bin/bash
# Equibop launcher with overlay daemon

# Kill any existing daemon
pkill -f discord-overlay-daemon 2>/dev/null

# Start overlay daemon in background
DAEMON_PATH=""
if [ -f "$HOME/.local/bin/discord-overlay-daemon" ]; then
    DAEMON_PATH="$HOME/.local/bin/discord-overlay-daemon"
elif [ -f "/usr/bin/discord-overlay-daemon" ]; then
    DAEMON_PATH="/usr/bin/discord-overlay-daemon"
fi

if [ -n "$DAEMON_PATH" ]; then
    echo "[Equibop Overlay] Starting daemon: $DAEMON_PATH"
    GDK_BACKEND=wayland "$DAEMON_PATH" &
    DAEMON_PID=$!
    echo "[Equibop Overlay] Daemon started with PID $DAEMON_PID"

    # Cleanup function
    cleanup() {
        echo "[Equibop Overlay] Stopping daemon"
        kill $DAEMON_PID 2>/dev/null
    }
    trap cleanup EXIT
else
    echo "[Equibop Overlay] Warning: daemon not found"
fi

# Launch Equibop
exec /usr/bin/equibop "$@"
EOF
    chmod +x ~/.local/bin/equibop-overlay
    echo ""
    echo "Wrapper created! Launch with: equibop-overlay"
fi

echo ""
echo "=== Installation Complete! ==="
echo ""
echo "The daemon has been installed to: ~/.local/bin/discord-overlay-daemon"
echo ""
echo "Usage:"
echo "  1. Manual: GDK_BACKEND=wayland discord-overlay-daemon"
if command -v equibop &> /dev/null; then
    echo "  2. With Equibop: equibop-overlay"
fi
echo ""
echo "Make sure OrbolayBridge plugin is enabled in Equibop!"
