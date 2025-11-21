use serde::{Deserialize, Serialize};

/// Represents a user in a Discord voice channel (Orbolay protocol)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VoiceUser {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub username: String,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    #[serde(rename = "channelId")]
    pub channel_id: Option<String>,
    pub deaf: bool,
    pub mute: bool,
    pub streaming: bool,
    pub speaking: bool,
}

impl VoiceUser {
    /// Returns the full avatar URL
    pub fn full_avatar_url(&self) -> Option<String> {
        self.avatar_url.as_ref().map(|hash| {
            if hash.starts_with("http") {
                hash.clone()
            } else {
                format!(
                    "https://cdn.discordapp.com/avatars/{}/{}.png?size=64",
                    self.user_id, hash
                )
            }
        })
    }

    /// Returns initials for users without avatars
    pub fn initials(&self) -> String {
        self.username
            .split_whitespace()
            .filter_map(|word| word.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    }
}

/// Config message from Orbolay plugin
#[derive(Debug, Clone, Deserialize)]
pub struct ConfigMessage {
    pub cmd: String,
    pub port: Option<u16>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "messageAlignment")]
    pub message_alignment: Option<String>,
    #[serde(rename = "userAlignment")]
    pub user_alignment: Option<String>,
    #[serde(rename = "voiceSemitransparent")]
    pub voice_semitransparent: Option<bool>,
    #[serde(rename = "messagesSemitransparent")]
    pub messages_semitransparent: Option<bool>,
}

/// Channel joined message
#[derive(Debug, Clone, Deserialize)]
pub struct ChannelJoinedMessage {
    pub cmd: String,
    pub states: Vec<VoiceUser>,
    #[serde(rename = "channelName")]
    pub channel_name: Option<String>,
}

/// Voice state update message
#[derive(Debug, Clone, Deserialize)]
pub struct VoiceStateUpdateMessage {
    pub cmd: String,
    pub state: VoiceUserPartial,
}

/// Partial voice user update (for speaking/state changes)
#[derive(Debug, Clone, Deserialize)]
pub struct VoiceUserPartial {
    #[serde(rename = "userId")]
    pub user_id: String,
    pub username: Option<String>,
    #[serde(rename = "avatarUrl")]
    pub avatar_url: Option<String>,
    #[serde(rename = "channelId")]
    pub channel_id: Option<String>,
    pub deaf: Option<bool>,
    pub mute: Option<bool>,
    pub streaming: Option<bool>,
    pub speaking: Option<bool>,
}

/// Message notification
#[derive(Debug, Clone, Deserialize)]
pub struct MessageNotification {
    pub cmd: String,
    pub message: NotificationContent,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NotificationContent {
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    #[serde(rename = "channelId")]
    pub channel_id: Option<String>,
}

/// Generic incoming message (for parsing cmd field first)
#[derive(Debug, Clone, Deserialize)]
pub struct GenericMessage {
    pub cmd: String,
}

/// Events sent to the UI
#[derive(Debug, Clone)]
pub enum OverlayEvent {
    ChannelJoined(Vec<VoiceUser>, String), // users, channel_name
    ChannelLeft,
    VoiceStateUpdate(VoiceUserPartial),
    ConfigReceived(ConfigMessage),
    MessageNotification(NotificationContent),
}
