mod config;

use config::{Config, Position};
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Application, ApplicationWindow, Box as GtkBox, Button, CheckButton,
    ComboBoxText, Label, Orientation, Scale, SpinButton,
};

const APP_ID: &str = "com.discord.overlay.config";

fn main() -> gtk4::glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let config = Config::load();

    let window = ApplicationWindow::builder()
        .application(app)
        .title("Discord Overlay - Configuration")
        .default_width(400)
        .default_height(350)
        .resizable(false)
        .build();

    let main_box = GtkBox::new(Orientation::Vertical, 12);
    main_box.set_margin_top(20);
    main_box.set_margin_bottom(20);
    main_box.set_margin_start(20);
    main_box.set_margin_end(20);

    // Title
    let title = Label::new(Some("Discord Overlay Configuration"));
    title.add_css_class("title-2");
    main_box.append(&title);

    // Position selector
    let pos_box = GtkBox::new(Orientation::Horizontal, 8);
    let pos_label = Label::new(Some("Position:"));
    pos_label.set_width_chars(12);
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
    main_box.append(&pos_box);

    // Margin
    let margin_box = GtkBox::new(Orientation::Horizontal, 8);
    let margin_label = Label::new(Some("Margin (px):"));
    margin_label.set_width_chars(12);
    margin_label.set_xalign(0.0);
    let margin_adj = Adjustment::new(config.margin as f64, 0.0, 200.0, 1.0, 10.0, 0.0);
    let margin_spin = SpinButton::new(Some(&margin_adj), 1.0, 0);
    margin_spin.set_hexpand(true);
    margin_box.append(&margin_label);
    margin_box.append(&margin_spin);
    main_box.append(&margin_box);

    // Opacity
    let opacity_box = GtkBox::new(Orientation::Horizontal, 8);
    let opacity_label = Label::new(Some("Opacity:"));
    opacity_label.set_width_chars(12);
    opacity_label.set_xalign(0.0);
    let opacity_adj = Adjustment::new(config.opacity, 0.1, 1.0, 0.05, 0.1, 0.0);
    let opacity_scale = Scale::new(Orientation::Horizontal, Some(&opacity_adj));
    opacity_scale.set_hexpand(true);
    opacity_scale.set_draw_value(true);
    opacity_scale.set_digits(2);
    opacity_box.append(&opacity_label);
    opacity_box.append(&opacity_scale);
    main_box.append(&opacity_box);

    // Avatar size
    let avatar_box = GtkBox::new(Orientation::Horizontal, 8);
    let avatar_label = Label::new(Some("Avatar size:"));
    avatar_label.set_width_chars(12);
    avatar_label.set_xalign(0.0);
    let avatar_adj = Adjustment::new(config.avatar_size as f64, 16.0, 64.0, 4.0, 8.0, 0.0);
    let avatar_spin = SpinButton::new(Some(&avatar_adj), 1.0, 0);
    avatar_spin.set_hexpand(true);
    avatar_box.append(&avatar_label);
    avatar_box.append(&avatar_spin);
    main_box.append(&avatar_box);

    // Port
    let port_box = GtkBox::new(Orientation::Horizontal, 8);
    let port_label = Label::new(Some("WebSocket port:"));
    port_label.set_width_chars(12);
    port_label.set_xalign(0.0);
    let port_adj = Adjustment::new(config.port as f64, 1024.0, 65535.0, 1.0, 100.0, 0.0);
    let port_spin = SpinButton::new(Some(&port_adj), 1.0, 0);
    port_spin.set_hexpand(true);
    port_box.append(&port_label);
    port_box.append(&port_spin);
    main_box.append(&port_box);

    // Show header checkbox
    let header_check = CheckButton::with_label("Show 'Voice Connected' header");
    header_check.set_active(config.show_header);
    main_box.append(&header_check);

    // Buttons
    let button_box = GtkBox::new(Orientation::Horizontal, 8);
    button_box.set_halign(gtk4::Align::End);
    button_box.set_margin_top(12);

    let save_btn = Button::with_label("Save");
    save_btn.add_css_class("suggested-action");

    let cancel_btn = Button::with_label("Cancel");

    button_box.append(&cancel_btn);
    button_box.append(&save_btn);
    main_box.append(&button_box);

    // Connect signals
    let window_clone = window.clone();
    cancel_btn.connect_clicked(move |_| {
        window_clone.close();
    });

    let window_clone = window.clone();
    save_btn.connect_clicked(move |_| {
        let position = match pos_combo.active_id().as_deref() {
            Some("top-left") => Position::TopLeft,
            Some("bottom-right") => Position::BottomRight,
            Some("bottom-left") => Position::BottomLeft,
            _ => Position::TopRight,
        };

        let new_config = Config {
            position,
            margin: margin_spin.value() as i32,
            opacity: opacity_scale.value(),
            port: port_spin.value() as u16,
            show_header: header_check.is_active(),
            avatar_size: avatar_spin.value() as i32,
        };

        new_config.save();
        window_clone.close();
    });

    window.set_child(Some(&main_box));
    window.present();
}
