use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, ApplicationWindow};
use gtk4::glib;
use std::cell::RefCell;
use std::rc::Rc;

use crate::discord_data::NotificationContent;
use crate::config::Config;

/// Notification window positioned at bottom right
pub struct NotificationWindow {
    window: ApplicationWindow,
    container: GtkBox,
    notifications: Rc<RefCell<Vec<GtkBox>>>,
}

impl NotificationWindow {
    pub fn new(app: &gtk4::Application, config: &Config) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Discord Notifications")
            .decorated(false)
            .resizable(false)
            .build();

        // Setup layer-shell for notifications window
        Self::setup_notification_layer_shell(&window, config);

        let container = GtkBox::new(Orientation::Vertical, 4);
        container.add_css_class("notification-window");
        container.set_valign(Align::End);
        container.set_halign(Align::End);

        window.set_child(Some(&container));

        // Initially hidden until first notification
        window.set_visible(false);

        Self {
            window,
            container,
            notifications: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn setup_notification_layer_shell(window: &ApplicationWindow, config: &Config) {
        use gtk4_layer_shell::{Edge, Layer, LayerShell};

        window.init_layer_shell();
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(gtk4_layer_shell::KeyboardMode::None);

        // Anchor to bottom right
        window.set_anchor(Edge::Bottom, true);
        window.set_anchor(Edge::Right, true);
        window.set_anchor(Edge::Top, false);
        window.set_anchor(Edge::Left, false);

        // Margins
        window.set_margin(Edge::Bottom, 20);
        window.set_margin(Edge::Right, 20);

        // Click-through - let mouse events pass through overlay
        if config.click_through {
            window.set_can_target(false);
        }

        // Use notifications namespace
        window.set_namespace("discord-overlay-notifications");
    }

    pub fn show_notification(&mut self, notif: NotificationContent) {
        // Create notification widget
        let notif_box = GtkBox::new(Orientation::Vertical, 4);
        notif_box.add_css_class("notification");

        let title = Label::new(Some(&notif.title));
        title.add_css_class("notification-title");
        title.set_halign(Align::Start);
        notif_box.append(&title);

        let body = Label::new(Some(&notif.body));
        body.add_css_class("notification-body");
        body.set_wrap(true);
        body.set_max_width_chars(35);
        body.set_halign(Align::Start);
        notif_box.append(&body);

        self.container.append(&notif_box);
        self.notifications.borrow_mut().push(notif_box.clone());

        // Show window if hidden
        if !self.window.is_visible() {
            self.window.set_visible(true);
        }

        // Auto-remove after 5 seconds
        let container_clone = self.container.clone();
        let notifications_clone = self.notifications.clone();
        let window_clone = self.window.clone();

        glib::timeout_add_seconds_local_once(5, move || {
            container_clone.remove(&notif_box);

            // Remove from tracking list
            if let Ok(mut notifs) = notifications_clone.try_borrow_mut() {
                notifs.retain(|n| n != &notif_box);

                // Hide window if no more notifications
                if notifs.is_empty() {
                    window_clone.set_visible(false);
                }
            }
        });
    }

    pub fn present(&self) {
        self.window.present();
    }
}
