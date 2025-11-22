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
        window.set_can_target(false);
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
