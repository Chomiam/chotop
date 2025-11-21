use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

/// Overlay position on screen
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Default)]
pub enum Position {
    #[default]
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
}

/// Configuration for the overlay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Position on screen
    pub position: Position,
    /// Margin from screen edge (pixels)
    pub margin: i32,
    /// Opacity (0.0 - 1.0)
    pub opacity: f64,
    /// WebSocket port
    pub port: u16,
    /// Show header "Voice Connected"
    pub show_header: bool,
    /// Avatar size in pixels
    pub avatar_size: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            position: Position::TopRight,
            margin: 20,
            opacity: 0.9,
            port: 6888,
            show_header: true,
            avatar_size: 32,
        }
    }
}

impl Config {
    /// Get config file path
    pub fn config_path() -> PathBuf {
        let config_dir = std::env::var("XDG_CONFIG_HOME")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                std::env::var("HOME")
                    .ok()
                    .map(|h| PathBuf::from(h).join(".config"))
            })
            .unwrap_or_else(|| PathBuf::from("."));

        config_dir.join("discord-overlay").join("config.toml")
    }

    /// Load config from file or create default
    pub fn load() -> Self {
        let path = Self::config_path();

        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => {
                        info!("Config loaded from {:?}", path);
                        return config;
                    }
                    Err(e) => {
                        warn!("Failed to parse config: {}", e);
                    }
                },
                Err(e) => {
                    warn!("Failed to read config file: {}", e);
                }
            }
        }

        let config = Self::default();
        config.save();
        config
    }

    /// Save config to file
    pub fn save(&self) {
        let path = Self::config_path();

        // Create parent directory
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        match toml::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = fs::write(&path, content) {
                    warn!("Failed to save config: {}", e);
                } else {
                    info!("Config saved to {:?}", path);
                }
            }
            Err(e) => {
                warn!("Failed to serialize config: {}", e);
            }
        }
    }
}
