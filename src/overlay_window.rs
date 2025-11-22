use gtk4::gdk;
use gtk4::prelude::*;
use gtk4::{ApplicationWindow, CssProvider};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

use crate::config::{Config, Position};

/// Configure the window as a Wayland layer-shell overlay
pub fn setup_layer_shell(window: &ApplicationWindow, config: &Config) {
    // Initialize layer shell for this window
    window.init_layer_shell();

    // Set layer to Overlay (above everything else)
    window.set_layer(Layer::Overlay);

    // Anchor based on position
    match config.position {
        Position::TopRight => {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Right, true);
            window.set_anchor(Edge::Bottom, false);
            window.set_anchor(Edge::Left, false);
            window.set_margin(Edge::Top, config.margin);
            window.set_margin(Edge::Right, config.margin);
        }
        Position::TopLeft => {
            window.set_anchor(Edge::Top, true);
            window.set_anchor(Edge::Left, true);
            window.set_anchor(Edge::Bottom, false);
            window.set_anchor(Edge::Right, false);
            window.set_margin(Edge::Top, config.margin);
            window.set_margin(Edge::Left, config.margin);
        }
        Position::BottomRight => {
            window.set_anchor(Edge::Bottom, true);
            window.set_anchor(Edge::Right, true);
            window.set_anchor(Edge::Top, false);
            window.set_anchor(Edge::Left, false);
            window.set_margin(Edge::Bottom, config.margin);
            window.set_margin(Edge::Right, config.margin);
        }
        Position::BottomLeft => {
            window.set_anchor(Edge::Bottom, true);
            window.set_anchor(Edge::Left, true);
            window.set_anchor(Edge::Top, false);
            window.set_anchor(Edge::Right, false);
            window.set_margin(Edge::Bottom, config.margin);
            window.set_margin(Edge::Left, config.margin);
        }
    }

    // Keyboard passthrough - let keys go to underlying windows
    window.set_keyboard_mode(KeyboardMode::None);

    // Click-through - let mouse events pass through overlay
    if config.click_through {
        // Set empty input region to make window click-through
        // Must be done after window is realized (surface exists)
        let window_clone = window.clone();
        window.connect_realize(move |_| {
            if let Some(surface) = window_clone.surface() {
                let empty_region = gdk::cairo::Region::create();
                surface.set_input_region(&empty_region);
            }
        });
    }

    // Set namespace for compositor identification
    window.set_namespace("discord-overlay");
}

/// Load and apply CSS styles for the overlay
pub fn load_css(config: &Config) {
    let provider = CssProvider::new();

    // Generate CSS with config values
    let css = format!(
        r#"
/* Discord Overlay - Dark Theme */

window {{
    background-color: transparent;
}}

.overlay-container {{
    background-color: rgba(30, 31, 34, {opacity});
    border-radius: 8px;
    padding: 8px;
    min-width: 180px;
}}

.user-row {{
    padding: 6px 8px;
    border-radius: 4px;
    margin: 2px 0;
}}

.user-row:hover {{
    background-color: rgba(79, 84, 92, 0.4);
}}

/* Avatar styling */
.avatar {{
    border-radius: 50%;
    min-width: {avatar_size}px;
    min-height: {avatar_size}px;
}}

.avatar-frame {{
    border-radius: 50%;
    padding: 2px;
    background-color: transparent;
}}

.avatar-frame.speaking {{
    background-color: #23a55a;
}}

/* Avatar placeholder for users without avatars */
.avatar-placeholder {{
    background-color: #5865f2;
    border-radius: 50%;
    min-width: {avatar_size}px;
    min-height: {avatar_size}px;
}}

.avatar-initials {{
    color: white;
    font-weight: bold;
    font-size: 12px;
}}

/* Username styling */
.username {{
    color: #f2f3f5;
    font-size: 13px;
    font-weight: 500;
    margin-left: 8px;
}}

.username.speaking {{
    color: #23a55a;
}}

.username.muted {{
    color: #949ba4;
}}

/* Status icons */
.status-icons {{
    margin-left: auto;
    padding-left: 8px;
}}

.status-icon {{
    color: #ed4245;
    font-size: 14px;
    margin-left: 4px;
}}

.status-icon.muted {{
    color: #ed4245;
}}

.status-icon.deafened {{
    color: #ed4245;
}}

.status-icon.streaming {{
    color: #23a55a;
}}

/* Empty state */
.empty-state {{
    color: #949ba4;
    font-size: 12px;
    padding: 16px;
    font-style: italic;
}}

/* Header */
.overlay-header {{
    color: #949ba4;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 4px 8px 8px 8px;
    border-bottom: 1px solid rgba(79, 84, 92, 0.6);
    margin-bottom: 4px;
}}

/* Notification Window */
.notification-window {{
    background-color: transparent;
    padding: 8px;
}}

.notification {{
    background-color: rgba(30, 31, 34, 0.95);
    border: 1px solid rgba(88, 101, 242, 0.4);
    border-left: 4px solid #5865f2;
    border-radius: 8px;
    padding: 14px 16px;
    margin: 6px 0;
    min-width: 320px;
    max-width: 380px;
    animation: slideInRight 0.3s ease-out;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.5);
    transition: all 0.2s ease;
}}

.notification.clickable:hover {{
    background-color: rgba(40, 41, 44, 0.98);
    border-left-color: #7289da;
    box-shadow: 0 6px 20px rgba(88, 101, 242, 0.3);
    transform: translateY(-2px);
    cursor: pointer;
}}

@keyframes slideInRight {{
    from {{
        opacity: 0;
        transform: translateX(20px);
    }}
    to {{
        opacity: 1;
        transform: translateX(0);
    }}
}}

/* Notification Avatar */
.notification-avatar {{
    border-radius: 50%;
    min-width: 48px;
    min-height: 48px;
    max-width: 48px;
    max-height: 48px;
    border: 2px solid rgba(88, 101, 242, 0.3);
}}

.notification-avatar-placeholder {{
    background: linear-gradient(135deg, #5865f2 0%, #7289da 100%);
    border-radius: 50%;
    min-width: 48px;
    min-height: 48px;
    border: 2px solid rgba(88, 101, 242, 0.5);
}}

.notification-avatar-initials {{
    color: white;
    font-weight: 700;
    font-size: 18px;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.3);
}}

/* Notification Content */
.notification-title {{
    color: #ffffff;
    font-size: 15px;
    font-weight: 700;
    margin-bottom: 4px;
    text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
}}

.notification-body {{
    color: #dcddde;
    font-size: 13px;
    line-height: 1.5;
    opacity: 0.95;
}}
"#,
        opacity = config.opacity,
        avatar_size = config.avatar_size,
    );

    provider.load_from_data(&css);

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not get default display"),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
