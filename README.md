# Chotop

Discord voice overlay daemon for Wayland with GTK4 and layer-shell support.

## Features

- 🎮 **Voice Channel Overlay** - Real-time display of voice channel participants
- 👤 **User Status** - Shows avatars, speaking indicators, mute/deaf/streaming status
- 📢 **Message Notifications** - Separate window for Discord message notifications (bottom-right)
- 📺 **Channel Name Display** - Shows current voice channel name
- 🔌 **Orbolay Compatible** - Works with OrbolayBridge plugin in Equicord/Equibop
- 🪟 **Native Wayland** - Uses GTK4 layer-shell for true overlay support
- 🖱️ **Click-Through** - Mouse events pass through overlay to underlying windows (configurable)
- ⚙️ **Configurable** - Position, opacity, and other settings

## Screenshots

*Coming soon*

## Dependencies

### Arch Linux
```bash
sudo pacman -S gtk4 gtk4-layer-shell rust
```

### Debian/Ubuntu
```bash
sudo apt install libgtk-4-dev libgtk4-layer-shell-dev cargo
```

## Installation

### Quick Install
```bash
git clone https://github.com/Chomiam/chotop.git
cd chotop
chmod +x install.sh
./install.sh
```

### Manual Build
```bash
cargo build --release
sudo cp target/release/discord-overlay-daemon /usr/local/bin/
```

## Usage

### With Equibop (Recommended)

1. Install Equibop
2. Enable the **OrbolayBridge** plugin in Equibop settings
3. Launch Equibop using the wrapper:
   ```bash
   equibop-overlay
   ```

The daemon will automatically start and stop with Equibop!

### Standalone

```bash
GDK_BACKEND=wayland discord-overlay-daemon
```

Then start Equibop/Equicord with OrbolayBridge plugin enabled.

## Configuration

Config file location: `~/.config/discord-overlay/config.toml`

You can edit the config file manually or use the GUI configuration tool:
```bash
chotop-config
```

### Configuration Options

```toml
position = "TopLeft"  # TopLeft, TopRight, BottomLeft, BottomRight
margin = 20           # Distance from screen edge in pixels
opacity = 0.9         # Transparency (0.1 to 1.0)
port = 6888           # WebSocket port
avatar_size = 32      # Avatar size in pixels (16-64)
click_through = true  # Allow clicking through overlay (default: true)
```

**Click-Through Mode:**
- When `click_through = true` (default), mouse events pass through the overlay to windows below
- Set to `false` if you want to interact with the overlay (hover effects, etc.)
- Restart the daemon after changing this setting

## How it Works

1. **Chotop daemon** runs in the background and listens on WebSocket port 6888
2. **OrbolayBridge plugin** (in Equibop) sends voice state updates via WebSocket
3. **Two overlay windows** are displayed:
   - Voice channel participants (configurable position)
   - Message notifications (bottom-right)

## Troubleshooting

### Overlay not showing
- Make sure you're running on Wayland: `echo $XDG_SESSION_TYPE`
- Check if daemon is running: `pgrep discord-overlay-daemon`
- Verify OrbolayBridge plugin is enabled in Equibop

### Connection issues
- Default port is 6888, make sure it's not blocked
- Check daemon logs for WebSocket connection messages

## Credits

- Built with [GTK4](https://www.gtk.org/) and [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell)
- Compatible with [Orbolay protocol](https://github.com/OpenAsar/arrpc)
- Designed for [Equibop](https://github.com/Equicord/Equibop) and [Equicord](https://github.com/Equicord/Equicord)

## License

MIT License - see LICENSE file for details
