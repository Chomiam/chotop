use gtk4::prelude::*;
use gtk4::{
    Adjustment, Application, ApplicationWindow, Box as GtkBox, Button, CheckButton,
    ComboBoxText, Label, Orientation, Scale, SpinButton, Switch,
};
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{Config, Position};

pub struct ControlWindow {
    window: ApplicationWindow,
}

impl ControlWindow {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Chotop - Control Panel")
            .default_width(500)
            .default_height(600)
            .build();

        Self { window }
    }

    pub fn build_ui(
        &self,
        on_test_mode: Box<dyn Fn(bool) + 'static>,
        on_config_changed: Box<dyn Fn(Config) + 'static>,
        on_restart: Box<dyn Fn() + 'static>,
        on_quit: Box<dyn Fn() + 'static>,
    ) {
        let config = Rc::new(RefCell::new(Config::load()));

        let main_box = GtkBox::new(Orientation::Vertical, 16);
        main_box.set_margin_top(20);
        main_box.set_margin_bottom(20);
        main_box.set_margin_start(20);
        main_box.set_margin_end(20);

        // Header
        let header = Label::new(Some("Chotop Control Panel"));
        header.add_css_class("title-1");
        main_box.append(&header);

        let status = Label::new(Some("Overlay daemon is running"));
        status.add_css_class("dim-label");
        main_box.append(&status);

        // Test Mode Section
        let test_section = GtkBox::new(Orientation::Vertical, 8);
        let test_header = Label::new(Some("Test Mode"));
        test_header.add_css_class("title-3");
        test_header.set_xalign(0.0);
        test_section.append(&test_header);

        let test_box = GtkBox::new(Orientation::Horizontal, 8);
        let test_label = Label::new(Some("Show preview with test data:"));
        test_label.set_hexpand(true);
        test_label.set_xalign(0.0);
        let test_switch = Switch::new();
        test_switch.set_active(false);

        let on_test_mode = Rc::new(on_test_mode);
        let on_test_mode_clone = on_test_mode.clone();
        test_switch.connect_state_set(move |_, enabled| {
            on_test_mode_clone(enabled);
            gtk4::glib::Propagation::Proceed
        });

        test_box.append(&test_label);
        test_box.append(&test_switch);
        test_section.append(&test_box);
        main_box.append(&test_section);

        // Appearance Section
        let appearance_section = GtkBox::new(Orientation::Vertical, 8);
        let appearance_header = Label::new(Some("Appearance"));
        appearance_header.add_css_class("title-3");
        appearance_header.set_xalign(0.0);
        appearance_section.append(&appearance_header);

        // Position
        let pos_box = GtkBox::new(Orientation::Horizontal, 8);
        let pos_label = Label::new(Some("Position:"));
        pos_label.set_width_chars(15);
        pos_label.set_xalign(0.0);
        let pos_combo = ComboBoxText::new();
        pos_combo.append(Some("top-right"), "Top Right");
        pos_combo.append(Some("top-left"), "Top Left");
        pos_combo.append(Some("bottom-right"), "Bottom Right");
        pos_combo.append(Some("bottom-left"), "Bottom Left");
        pos_combo.set_active_id(Some(match config.borrow().position {
            Position::TopRight => "top-right",
            Position::TopLeft => "top-left",
            Position::BottomRight => "bottom-right",
            Position::BottomLeft => "bottom-left",
        }));
        pos_combo.set_hexpand(true);

        let config_clone = config.clone();
        let on_config_changed_clone = Rc::new(on_config_changed);
        let on_config_changed_pos = on_config_changed_clone.clone();
        pos_combo.connect_changed(move |combo| {
            if let Some(id) = combo.active_id() {
                let mut cfg = config_clone.borrow_mut();
                cfg.position = match id.as_str() {
                    "top-left" => Position::TopLeft,
                    "bottom-right" => Position::BottomRight,
                    "bottom-left" => Position::BottomLeft,
                    _ => Position::TopRight,
                };
                on_config_changed_pos(cfg.clone());
            }
        });

        pos_box.append(&pos_label);
        pos_box.append(&pos_combo);
        appearance_section.append(&pos_box);

        // Margin
        let margin_box = GtkBox::new(Orientation::Horizontal, 8);
        let margin_label = Label::new(Some("Margin (px):"));
        margin_label.set_width_chars(15);
        margin_label.set_xalign(0.0);
        let margin_adj = Adjustment::new(config.borrow().margin as f64, 0.0, 200.0, 1.0, 10.0, 0.0);
        let margin_spin = SpinButton::new(Some(&margin_adj), 1.0, 0);
        margin_spin.set_hexpand(true);

        let config_clone = config.clone();
        let on_config_changed_margin = on_config_changed_clone.clone();
        margin_spin.connect_value_changed(move |spin| {
            let mut cfg = config_clone.borrow_mut();
            cfg.margin = spin.value() as i32;
            on_config_changed_margin(cfg.clone());
        });

        margin_box.append(&margin_label);
        margin_box.append(&margin_spin);
        appearance_section.append(&margin_box);

        // Opacity
        let opacity_box = GtkBox::new(Orientation::Horizontal, 8);
        let opacity_label = Label::new(Some("Opacity:"));
        opacity_label.set_width_chars(15);
        opacity_label.set_xalign(0.0);
        let opacity_adj = Adjustment::new(config.borrow().opacity, 0.1, 1.0, 0.05, 0.1, 0.0);
        let opacity_scale = Scale::new(Orientation::Horizontal, Some(&opacity_adj));
        opacity_scale.set_hexpand(true);
        opacity_scale.set_draw_value(true);
        opacity_scale.set_digits(2);

        let config_clone = config.clone();
        let on_config_changed_opacity = on_config_changed_clone.clone();
        opacity_scale.connect_value_changed(move |scale| {
            let mut cfg = config_clone.borrow_mut();
            cfg.opacity = scale.value();
            on_config_changed_opacity(cfg.clone());
        });

        opacity_box.append(&opacity_label);
        opacity_box.append(&opacity_scale);
        appearance_section.append(&opacity_box);

        // Avatar size
        let avatar_box = GtkBox::new(Orientation::Horizontal, 8);
        let avatar_label = Label::new(Some("Avatar size (px):"));
        avatar_label.set_width_chars(15);
        avatar_label.set_xalign(0.0);
        let avatar_adj = Adjustment::new(config.borrow().avatar_size as f64, 16.0, 64.0, 4.0, 8.0, 0.0);
        let avatar_spin = SpinButton::new(Some(&avatar_adj), 1.0, 0);
        avatar_spin.set_hexpand(true);

        let config_clone = config.clone();
        let on_config_changed_avatar = on_config_changed_clone.clone();
        avatar_spin.connect_value_changed(move |spin| {
            let mut cfg = config_clone.borrow_mut();
            cfg.avatar_size = spin.value() as i32;
            on_config_changed_avatar(cfg.clone());
        });

        avatar_box.append(&avatar_label);
        avatar_box.append(&avatar_spin);
        appearance_section.append(&avatar_box);

        main_box.append(&appearance_section);

        // Actions Section
        let actions_section = GtkBox::new(Orientation::Vertical, 8);
        let actions_header = Label::new(Some("Actions"));
        actions_header.add_css_class("title-3");
        actions_header.set_xalign(0.0);
        actions_section.append(&actions_header);

        let buttons_box = GtkBox::new(Orientation::Horizontal, 8);
        buttons_box.set_homogeneous(true);

        let restart_btn = Button::with_label("Restart Daemon");
        let on_restart = Rc::new(on_restart);
        restart_btn.connect_clicked(move |_| {
            on_restart();
        });

        let quit_btn = Button::with_label("Quit");
        quit_btn.add_css_class("destructive-action");
        let on_quit = Rc::new(on_quit);
        quit_btn.connect_clicked(move |_| {
            on_quit();
        });

        buttons_box.append(&restart_btn);
        buttons_box.append(&quit_btn);
        actions_section.append(&buttons_box);

        main_box.append(&actions_section);

        self.window.set_child(Some(&main_box));
    }

    pub fn show(&self) {
        self.window.present();
    }

    pub fn hide(&self) {
        self.window.set_visible(false);
    }
}
