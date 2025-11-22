use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, ApplicationWindow, GestureClick, Picture};
use gtk4::glib;
use std::cell::RefCell;
use std::io::Read;
use std::rc::Rc;
use std::path::PathBuf;

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

    fn setup_notification_layer_shell(window: &ApplicationWindow, _config: &Config) {
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

        // DON'T apply click-through on notifications - we want to be able to click them!

        // Use notifications namespace
        window.set_namespace("discord-overlay-notifications");
    }

    pub fn show_notification(&mut self, notif: NotificationContent) {
        // Create notification container
        let notif_box = GtkBox::new(Orientation::Horizontal, 12);
        notif_box.add_css_class("notification");

        // Avatar container
        let avatar_box = GtkBox::new(Orientation::Vertical, 0);
        avatar_box.set_valign(Align::Start);

        if let Some(icon_url) = &notif.icon {
            // Try to load avatar from URL
            if let Ok(icon_path) = Self::download_avatar(icon_url) {
                let picture = Picture::for_filename(&icon_path);
                picture.add_css_class("notification-avatar");
                picture.set_size_request(48, 48);
                picture.set_can_shrink(false);
                avatar_box.append(&picture);
            } else {
                // Fallback to placeholder
                let placeholder = Self::create_avatar_placeholder(&notif.title);
                avatar_box.append(&placeholder);
            }
        } else {
            // No icon provided, use placeholder
            let placeholder = Self::create_avatar_placeholder(&notif.title);
            avatar_box.append(&placeholder);
        }

        notif_box.append(&avatar_box);

        // Content container (title + body)
        let content_box = GtkBox::new(Orientation::Vertical, 6);
        content_box.set_hexpand(true);

        let title = Label::new(Some(&notif.title));
        title.add_css_class("notification-title");
        title.set_halign(Align::Start);
        title.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        title.set_max_width_chars(30);
        content_box.append(&title);

        let body = Label::new(Some(&notif.body));
        body.add_css_class("notification-body");
        body.set_wrap(true);
        body.set_wrap_mode(gtk4::pango::WrapMode::WordChar);
        body.set_max_width_chars(30);
        body.set_lines(3);
        body.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        body.set_halign(Align::Start);
        body.set_valign(Align::Start);
        content_box.append(&body);

        notif_box.append(&content_box);

        // Add click handler to open Equibop
        let gesture = GestureClick::new();
        let channel_id = notif.channel_id.clone();
        gesture.connect_released(move |_gesture, _n_press, _x, _y| {
            tracing::info!("Notification clicked! Opening Equibop...");

            // Try to launch Equibop
            let mut command = std::process::Command::new("equibop");

            // If we have a channel ID, we could try to open it directly
            // Note: Discord/Equibop doesn't support direct channel opening via CLI
            // but we log it for future use or custom protocol handlers
            if let Some(ref channel) = channel_id {
                tracing::info!("Attempting to open channel: {}", channel);
                // Could implement a custom protocol handler here in the future
                // For now, just open Equibop normally
            }

            match command.spawn() {
                Ok(_) => tracing::info!("Equibop launched successfully"),
                Err(e) => tracing::error!("Failed to launch Equibop: {}", e),
            }
        });
        notif_box.add_controller(gesture);

        // Add hover effect
        notif_box.add_css_class("clickable");

        self.container.append(&notif_box);
        self.notifications.borrow_mut().push(notif_box.clone());

        // Show window if hidden
        if !self.window.is_visible() {
            self.window.set_visible(true);
        }

        // Auto-remove after 7 seconds
        let container_clone = self.container.clone();
        let notifications_clone = self.notifications.clone();
        let window_clone = self.window.clone();

        glib::timeout_add_seconds_local_once(7, move || {
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

    fn create_avatar_placeholder(username: &str) -> GtkBox {
        let placeholder_box = GtkBox::new(Orientation::Vertical, 0);
        placeholder_box.add_css_class("notification-avatar-placeholder");
        placeholder_box.set_valign(Align::Center);
        placeholder_box.set_halign(Align::Center);
        placeholder_box.set_size_request(48, 48);

        let initials = username
            .split_whitespace()
            .filter_map(|word| word.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase();

        let label = Label::new(Some(&initials));
        label.add_css_class("notification-avatar-initials");
        placeholder_box.append(&label);

        placeholder_box
    }

    fn download_avatar(url: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Simple synchronous download for notifications
        // In production, you'd want to use the avatar cache
        use std::io::Write;

        let response = ureq::get(url).call()?;
        let mut bytes = Vec::new();
        response.into_reader().read_to_end(&mut bytes)?;

        let cache_dir = std::env::var("XDG_CACHE_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".cache"))
            })
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("discord-overlay")
            .join("notification-avatars");

        std::fs::create_dir_all(&cache_dir)?;

        // Use hash of URL as filename
        let filename = format!("{:x}.png", md5::compute(url));
        let path = cache_dir.join(filename);

        // Check if already cached
        if path.exists() {
            return Ok(path);
        }

        let mut file = std::fs::File::create(&path)?;
        file.write_all(&bytes)?;

        Ok(path)
    }

    pub fn present(&self) {
        self.window.present();
    }
}
