mod config;

use config::{Config, Position};
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Application, ApplicationWindow, Box as GtkBox, Button,
    ComboBoxText, Label, Orientation, Scale, SpinButton,
};
use std::process::Command;

const APP_ID: &str = "com.discord.overlay.config";

fn is_daemon_running() -> bool {
    Command::new("pgrep")
        .arg("-f")
        .arg("discord-overlay-daemon")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn is_websocket_active() -> bool {
    use std::net::TcpStream;
    use std::time::Duration;

    TcpStream::connect_timeout(
        &"127.0.0.1:6888".parse().unwrap(),
        Duration::from_millis(100)
    ).is_ok()
}

fn is_ipc_active() -> bool {
    use std::path::Path;

    let socket_path = format!("/run/user/{}/chotop-control.sock",
        std::env::var("UID").unwrap_or_else(|_| "1000".to_string()));
    Path::new(&socket_path).exists()
}

fn main() -> gtk4::glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let config = Config::load();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Chotop - Configuration")
        .default_width(500)
        .default_height(600)
        .resizable(false)
        .build();

    let main_box = GtkBox::new(Orientation::Vertical, 16);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    // Header
    let header = Label::new(Some("Chotop Configuration"));
    header.add_css_class("title-1");
    main_box.append(&header);

    let subtitle = Label::new(Some("Discord Overlay for Wayland"));
    subtitle.add_css_class("dim-label");
    main_box.append(&subtitle);

    // Status indicators section
    let status_section = GtkBox::new(Orientation::Vertical, 4);
    status_section.set_margin_top(12);
    status_section.set_margin_bottom(12);

    // Daemon status
    let status_box = GtkBox::new(Orientation::Horizontal, 8);
    let status_indicator = Label::new(Some("●"));
    status_indicator.set_widget_name("status-indicator");
    let status_label = Label::new(None);
    status_label.set_widget_name("status-label");
    status_box.append(&status_indicator);
    status_box.append(&status_label);
    status_section.append(&status_box);

    // WebSocket status
    let ws_status_box = GtkBox::new(Orientation::Horizontal, 8);
    let ws_indicator = Label::new(Some("●"));
    ws_indicator.set_widget_name("ws-indicator");
    let ws_label = Label::new(None);
    ws_label.set_widget_name("ws-label");
    ws_status_box.append(&ws_indicator);
    ws_status_box.append(&ws_label);
    status_section.append(&ws_status_box);

    // IPC status
    let ipc_status_box = GtkBox::new(Orientation::Horizontal, 8);
    let ipc_indicator = Label::new(Some("●"));
    ipc_indicator.set_widget_name("ipc-indicator");
    let ipc_label = Label::new(None);
    ipc_label.set_widget_name("ipc-label");
    ipc_status_box.append(&ipc_indicator);
    ipc_status_box.append(&ipc_label);
    status_section.append(&ipc_status_box);

    main_box.append(&status_section);

    // Function to update status
    let update_status = {
        let status_indicator = status_indicator.clone();
        let status_label = status_label.clone();
        let ws_indicator = ws_indicator.clone();
        let ws_label = ws_label.clone();
        let ipc_indicator = ipc_indicator.clone();
        let ipc_label = ipc_label.clone();
        move || {
            // Daemon status
            if is_daemon_running() {
                status_indicator.set_markup("<span foreground='#4ade80' size='x-large'>●</span>");
                status_label.set_text("Daemon is running");
            } else {
                status_indicator.set_markup("<span foreground='#ef4444' size='x-large'>●</span>");
                status_label.set_text("Daemon is stopped");
            }

            // WebSocket status
            if is_websocket_active() {
                ws_indicator.set_markup("<span foreground='#4ade80' size='large'>●</span>");
                ws_label.set_text("WebSocket: Connected (port 6888)");
            } else {
                ws_indicator.set_markup("<span foreground='#ef4444' size='large'>●</span>");
                ws_label.set_text("WebSocket: Disconnected");
            }

            // IPC status
            if is_ipc_active() {
                ipc_indicator.set_markup("<span foreground='#4ade80' size='large'>●</span>");
                ipc_label.set_text("IPC: Active");
            } else {
                ipc_indicator.set_markup("<span foreground='#ef4444' size='large'>●</span>");
                ipc_label.set_text("IPC: Inactive");
            }
        }
    };

    // Initial status update
    update_status();

    // Periodic status check (every 2 seconds)
    let update_status_periodic = update_status.clone();
    gtk4::glib::timeout_add_seconds_local(2, move || {
        update_status_periodic();
        gtk4::glib::ControlFlow::Continue
    });

    // Appearance Section
    let appearance_section = GtkBox::new(Orientation::Vertical, 12);
    let appearance_header = Label::new(Some("Appearance"));
    appearance_header.add_css_class("title-2");
    appearance_header.set_xalign(0.0);
    appearance_section.append(&appearance_header);

    // Position
    let pos_box = GtkBox::new(Orientation::Horizontal, 12);
    let pos_label = Label::new(Some("Overlay Position:"));
    pos_label.set_width_chars(18);
    pos_label.set_xalign(0.0);
    let pos_combo = ComboBoxText::new();
    pos_combo.append(Some("top-right"), "Top Right");
    pos_combo.append(Some("top-left"), "Top Left");
    pos_combo.append(Some("bottom-right"), "Bottom Right");
    pos_combo.append(Some("bottom-left"), "Bottom Left");
    pos_combo.set_active_id(Some(match config.position {
        Position::TopRight => "top-right",
        Position::TopLeft => "top-left",
        Position::BottomRight => "bottom-right",
        Position::BottomLeft => "bottom-left",
    }));
    pos_combo.set_hexpand(true);
    pos_box.append(&pos_label);
    pos_box.append(&pos_combo);
    appearance_section.append(&pos_box);

    // Margin
    let margin_box = GtkBox::new(Orientation::Horizontal, 12);
    let margin_label = Label::new(Some("Margin (pixels):"));
    margin_label.set_width_chars(18);
    margin_label.set_xalign(0.0);
    let margin_adj = Adjustment::new(config.margin as f64, 0.0, 200.0, 1.0, 10.0, 0.0);
    let margin_spin = SpinButton::new(Some(&margin_adj), 1.0, 0);
    margin_spin.set_hexpand(true);
    margin_box.append(&margin_label);
    margin_box.append(&margin_spin);
    appearance_section.append(&margin_box);

    // Opacity
    let opacity_box = GtkBox::new(Orientation::Horizontal, 12);
    let opacity_label = Label::new(Some("Opacity:"));
    opacity_label.set_width_chars(18);
    opacity_label.set_xalign(0.0);
    let opacity_adj = Adjustment::new(config.opacity, 0.1, 1.0, 0.05, 0.1, 0.0);
    let opacity_scale = Scale::new(Orientation::Horizontal, Some(&opacity_adj));
    opacity_scale.set_hexpand(true);
    opacity_scale.set_draw_value(true);
    opacity_scale.set_digits(2);
    opacity_box.append(&opacity_label);
    opacity_box.append(&opacity_scale);
    appearance_section.append(&opacity_box);

    // Avatar size
    let avatar_box = GtkBox::new(Orientation::Horizontal, 12);
    let avatar_label = Label::new(Some("Avatar Size (pixels):"));
    avatar_label.set_width_chars(18);
    avatar_label.set_xalign(0.0);
    let avatar_adj = Adjustment::new(config.avatar_size as f64, 16.0, 64.0, 4.0, 8.0, 0.0);
    let avatar_spin = SpinButton::new(Some(&avatar_adj), 1.0, 0);
    avatar_spin.set_hexpand(true);
    avatar_box.append(&avatar_label);
    avatar_box.append(&avatar_spin);
    appearance_section.append(&avatar_box);

    main_box.append(&appearance_section);

    // Info Section
    let info_box = GtkBox::new(Orientation::Vertical, 8);
    let info_label = Label::new(Some("💡 Tip: Restart the daemon after changing settings"));
    info_label.add_css_class("dim-label");
    info_label.set_wrap(true);
    info_box.append(&info_label);
    main_box.append(&info_box);

    // Buttons
    let buttons_box = GtkBox::new(Orientation::Horizontal, 12);
    buttons_box.set_homogeneous(true);
    buttons_box.set_margin_top(16);

    let restart_btn = Button::with_label("Start/Restart Daemon");
    let update_status_for_restart = update_status.clone();
    restart_btn.connect_clicked(move |_| {
        use std::process::Command;

        // Kill existing daemon
        let _ = Command::new("pkill")
            .arg("-f")
            .arg("discord-overlay-daemon")
            .status();

        // Clone update_status for async closure
        let update_status_async = update_status_for_restart.clone();

        // Wait and restart in background task
        gtk4::glib::spawn_future_local(async move {
            use gtk4::glib;

            glib::timeout_future(std::time::Duration::from_millis(500)).await;

            // Start daemon with proper environment
            let daemon_path = "/home/chomiam/Projet/overlay-daemon/target/release/discord-overlay-daemon";
            let _ = Command::new(daemon_path)
                .env("GDK_BACKEND", "wayland")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();

            // Wait a bit for daemon to start, then update status
            glib::timeout_future(std::time::Duration::from_millis(500)).await;
            update_status_async();
        });
    });

    let cancel_btn = Button::with_label("Cancel");
    let window_clone = window.clone();
    cancel_btn.connect_clicked(move |_| {
        window_clone.close();
    });

    let save_btn = Button::with_label("Save");
    save_btn.add_css_class("suggested-action");

    let window_clone2 = window.clone();
    save_btn.connect_clicked(move |_| {
        // Get values from widgets
        let position = match pos_combo.active_id().as_ref().map(|s| s.as_str()) {
            Some("top-left") => Position::TopLeft,
            Some("bottom-right") => Position::BottomRight,
            Some("bottom-left") => Position::BottomLeft,
            _ => Position::TopRight,
        };

        let new_config = Config {
            position,
            margin: margin_spin.value() as i32,
            opacity: opacity_scale.value(),
            avatar_size: avatar_spin.value() as i32,
            port: config.port,
        };

        new_config.save();

        // Show success dialog
        let dialog = gtk4::MessageDialog::builder()
            .transient_for(&window_clone2)
            .modal(true)
            .buttons(gtk4::ButtonsType::Ok)
            .text("Configuration Saved")
            .secondary_text("Please restart the daemon for changes to take effect.")
            .build();

        dialog.connect_response(move |dialog, _| {
            dialog.close();
        });

        dialog.present();
    });

    buttons_box.append(&restart_btn);
    buttons_box.append(&cancel_btn);
    buttons_box.append(&save_btn);
    main_box.append(&buttons_box);

    window.set_child(Some(&main_box));
    window.present();
}
