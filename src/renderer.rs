use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Label, Orientation, Picture};
use gtk4::gdk_pixbuf::Pixbuf;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;

use crate::discord_data::{VoiceUser, VoiceUserPartial};

/// Message to request avatar loading
#[derive(Debug, Clone)]
pub struct AvatarRequest {
    pub user_id: String,
    pub avatar_hash: String,
}

/// Renders the voice overlay UI
pub struct OverlayRenderer {
    container: GtkBox,
    users_box: GtkBox,
    user_widgets: HashMap<String, UserWidget>,
    users: HashMap<String, VoiceUser>,
    avatar_tx: Option<mpsc::Sender<AvatarRequest>>,
}

struct UserWidget {
    row: GtkBox,
    avatar_frame: GtkBox,
    avatar_picture: Option<Picture>,
}

impl OverlayRenderer {
    pub fn new() -> Self {
        let container = GtkBox::new(Orientation::Vertical, 0);
        container.add_css_class("overlay-container");

        // Users container
        let users_box = GtkBox::new(Orientation::Vertical, 0);
        users_box.add_css_class("users-box");
        container.append(&users_box);

        // Initially hidden
        container.set_visible(false);

        Self {
            container,
            users_box,
            user_widgets: HashMap::new(),
            users: HashMap::new(),
            avatar_tx: None,
        }
    }

    pub fn set_avatar_sender(&mut self, tx: mpsc::Sender<AvatarRequest>) {
        self.avatar_tx = Some(tx);
    }

    pub fn widget(&self) -> &GtkBox {
        &self.container
    }

    /// Enable test mode with fake data
    pub fn enable_test_mode(&mut self) {
        // Create fake users for testing
        let test_users = vec![
            VoiceUser {
                user_id: "test1".to_string(),
                username: "Alice".to_string(),
                avatar_url: None,
                channel_id: Some("test-channel".to_string()),
                deaf: false,
                mute: false,
                streaming: false,
                speaking: true,
            },
            VoiceUser {
                user_id: "test2".to_string(),
                username: "Bob".to_string(),
                avatar_url: None,
                channel_id: Some("test-channel".to_string()),
                deaf: false,
                mute: true,
                streaming: false,
                speaking: false,
            },
            VoiceUser {
                user_id: "test3".to_string(),
                username: "Charlie".to_string(),
                avatar_url: None,
                channel_id: Some("test-channel".to_string()),
                deaf: true,
                mute: false,
                streaming: true,
                speaking: false,
            },
        ];

        self.on_channel_joined(test_users, "Test Channel".to_string());
    }

    /// Disable test mode
    pub fn disable_test_mode(&mut self) {
        self.on_channel_left();
    }

    /// Update avatar for a user
    pub fn set_avatar(&mut self, user_id: &str, path: &PathBuf) {
        if let Some(user_widget) = self.user_widgets.get_mut(user_id) {
            // Load pixbuf and create picture
            if let Ok(pixbuf) = Pixbuf::from_file_at_scale(path, 32, 32, true) {
                // Remove old content from avatar frame
                while let Some(child) = user_widget.avatar_frame.first_child() {
                    user_widget.avatar_frame.remove(&child);
                }

                let picture = Picture::for_pixbuf(&pixbuf);
                picture.add_css_class("avatar");
                picture.set_size_request(32, 32);
                user_widget.avatar_frame.append(&picture);
                user_widget.avatar_picture = Some(picture);
            }
        }
    }

    /// Handle channel joined - set all users
    pub fn on_channel_joined(&mut self, users: Vec<VoiceUser>, _channel_name: String) {
        // Clear existing
        self.clear();

        // Add all users
        for user in users {
            self.add_user(user);
        }

        self.container.set_visible(!self.users.is_empty());
    }

    /// Handle channel left - clear all users
    pub fn on_channel_left(&mut self) {
        self.clear();
        self.container.set_visible(false);
    }

    /// Handle voice state update
    pub fn on_voice_state_update(&mut self, update: VoiceUserPartial) {
        tracing::debug!("Voice state update: user_id={}, channel_id={:?}, username={:?}",
            update.user_id, update.channel_id, update.username);

        // Check if user left (channel_id is None or empty)
        let user_left = match &update.channel_id {
            None => true,  // No channel_id means disconnected
            Some(ch) => ch.is_empty(),  // Empty string also means disconnected
        };

        if user_left {
            tracing::info!("User {} left the voice channel", update.user_id);
            self.remove_user(&update.user_id);
            self.container.set_visible(!self.users.is_empty());
            return;
        }

        if let Some(user) = self.users.get_mut(&update.user_id) {
            // Update existing user
            if let Some(speaking) = update.speaking {
                user.speaking = speaking;
            }
            if let Some(mute) = update.mute {
                user.mute = mute;
            }
            if let Some(deaf) = update.deaf {
                user.deaf = deaf;
            }
            if let Some(streaming) = update.streaming {
                user.streaming = streaming;
            }

            // Update widget
            if let Some(user_widget) = self.user_widgets.get(&update.user_id) {
                Self::update_user_widget(&user_widget.row, user);
            }
        } else if update.username.is_some() {
            // New user joined
            tracing::info!("User {} joined the voice channel", update.user_id);
            let user = VoiceUser {
                user_id: update.user_id.clone(),
                username: update.username.unwrap_or_default(),
                avatar_url: update.avatar_url,
                channel_id: update.channel_id,
                deaf: update.deaf.unwrap_or(false),
                mute: update.mute.unwrap_or(false),
                streaming: update.streaming.unwrap_or(false),
                speaking: update.speaking.unwrap_or(false),
            };
            self.add_user(user);
        }

        self.container.set_visible(!self.users.is_empty());
    }

    fn clear(&mut self) {
        for (_, user_widget) in self.user_widgets.drain() {
            self.users_box.remove(&user_widget.row);
        }
        self.users.clear();
    }

    fn add_user(&mut self, user: VoiceUser) {
        let user_widget = self.create_user_widget(&user);
        self.users_box.append(&user_widget.row);

        // Request avatar download if available
        if let (Some(tx), Some(avatar_hash)) = (&self.avatar_tx, &user.avatar_url) {
            let request = AvatarRequest {
                user_id: user.user_id.clone(),
                avatar_hash: avatar_hash.clone(),
            };
            let tx = tx.clone();
            gtk4::glib::spawn_future_local(async move {
                let _ = tx.send(request).await;
            });
        }

        self.user_widgets.insert(user.user_id.clone(), user_widget);
        self.users.insert(user.user_id.clone(), user);
    }

    fn remove_user(&mut self, user_id: &str) {
        if let Some(user_widget) = self.user_widgets.remove(user_id) {
            self.users_box.remove(&user_widget.row);
        }
        self.users.remove(user_id);
    }

    fn create_user_widget(&self, user: &VoiceUser) -> UserWidget {
        let row = GtkBox::new(Orientation::Horizontal, 0);
        row.add_css_class("user-row");
        row.set_widget_name(&user.user_id);

        // Avatar frame (for speaking indicator)
        let avatar_frame = GtkBox::new(Orientation::Vertical, 0);
        avatar_frame.add_css_class("avatar-frame");
        avatar_frame.set_widget_name("avatar-frame");
        avatar_frame.set_valign(Align::Center);

        // Avatar placeholder with initials (will be replaced by actual avatar)
        let initials_box = GtkBox::new(Orientation::Vertical, 0);
        initials_box.add_css_class("avatar-placeholder");
        initials_box.set_valign(Align::Center);
        initials_box.set_halign(Align::Center);

        let initials = Label::new(Some(&user.initials()));
        initials.add_css_class("avatar-initials");
        initials.set_valign(Align::Center);
        initials.set_halign(Align::Center);
        initials_box.append(&initials);

        avatar_frame.append(&initials_box);
        row.append(&avatar_frame);

        // Username
        let username = Label::new(Some(&user.username));
        username.add_css_class("username");
        username.set_widget_name("username");
        username.set_halign(Align::Start);
        username.set_hexpand(true);
        username.set_ellipsize(gtk4::pango::EllipsizeMode::End);
        username.set_max_width_chars(15);
        row.append(&username);

        // Status icons container
        let status_box = GtkBox::new(Orientation::Horizontal, 2);
        status_box.add_css_class("status-icons");
        status_box.set_widget_name("status-icons");
        row.append(&status_box);

        // Apply initial state
        Self::update_user_widget(&row, user);

        UserWidget {
            row,
            avatar_frame,
            avatar_picture: None,
        }
    }

    fn update_user_widget(widget: &GtkBox, user: &VoiceUser) {
        // Update speaking state on avatar frame
        let mut child = widget.first_child();
        while let Some(c) = child {
            let name = c.widget_name();

            if name == "avatar-frame" {
                if let Some(frame) = c.downcast_ref::<GtkBox>() {
                    if user.speaking {
                        frame.add_css_class("speaking");
                    } else {
                        frame.remove_css_class("speaking");
                    }
                }
            } else if name == "username" {
                if let Some(label) = c.downcast_ref::<Label>() {
                    if user.speaking {
                        label.add_css_class("speaking");
                    } else {
                        label.remove_css_class("speaking");
                    }
                    if user.mute || user.deaf {
                        label.add_css_class("muted");
                    } else {
                        label.remove_css_class("muted");
                    }
                }
            } else if name == "status-icons" {
                if let Some(status_box) = c.downcast_ref::<GtkBox>() {
                    // Clear existing icons
                    while let Some(icon) = status_box.first_child() {
                        status_box.remove(&icon);
                    }

                    // Add muted icon
                    if user.mute && !user.deaf {
                        let icon = Label::new(Some("🔇"));
                        icon.add_css_class("status-icon");
                        icon.add_css_class("muted");
                        status_box.append(&icon);
                    }

                    // Add deafened icon
                    if user.deaf {
                        let icon = Label::new(Some("🔕"));
                        icon.add_css_class("status-icon");
                        icon.add_css_class("deafened");
                        status_box.append(&icon);
                    }

                    // Add streaming icon
                    if user.streaming {
                        let icon = Label::new(Some("📺"));
                        icon.add_css_class("status-icon");
                        icon.add_css_class("streaming");
                        status_box.append(&icon);
                    }
                }
            }

            child = c.next_sibling();
        }
    }
}

impl Default for OverlayRenderer {
    fn default() -> Self {
        Self::new()
    }
}
