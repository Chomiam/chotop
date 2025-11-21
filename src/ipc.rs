use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio_tungstenite::accept_async;
use futures_util::StreamExt;
use tracing::{info, warn, error};

use crate::discord_data::{
    GenericMessage, ChannelJoinedMessage, VoiceStateUpdateMessage,
    ConfigMessage, MessageNotification, OverlayEvent,
};

/// WebSocket server that receives voice state updates from OrbolayBridge plugin
pub struct WebSocketServer {
    port: u16,
}

impl WebSocketServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    /// Start the WebSocket server and send events through the channel
    pub async fn run(self, tx: mpsc::Sender<OverlayEvent>) {
        let addr = format!("127.0.0.1:{}", self.port);

        let listener = match TcpListener::bind(&addr).await {
            Ok(l) => {
                info!("WebSocket server listening on {}", addr);
                l
            }
            Err(e) => {
                error!("Failed to bind WebSocket server: {}", e);
                return;
            }
        };

        loop {
            match listener.accept().await {
                Ok((stream, peer)) => {
                    info!("New connection from: {}", peer);
                    let tx_clone = tx.clone();

                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, tx_clone).await {
                            warn!("Connection error: {}", e);
                        }
                        info!("Connection closed: {}", peer);
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }
}

async fn handle_connection(
    stream: tokio::net::TcpStream,
    tx: mpsc::Sender<OverlayEvent>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ws_stream = accept_async(stream).await?;
    let (_, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(message) => {
                if message.is_text() {
                    let text = message.to_text()?;

                    // First parse to get the command type
                    match serde_json::from_str::<GenericMessage>(text) {
                        Ok(generic) => {
                            let event = match generic.cmd.as_str() {
                                "REGISTER_CONFIG" => {
                                    serde_json::from_str::<ConfigMessage>(text)
                                        .ok()
                                        .map(OverlayEvent::ConfigReceived)
                                }
                                "CHANNEL_JOINED" => {
                                    serde_json::from_str::<ChannelJoinedMessage>(text)
                                        .ok()
                                        .map(|m| OverlayEvent::ChannelJoined(
                                            m.states,
                                            m.channel_name.unwrap_or_else(|| "Voice Channel".to_string())
                                        ))
                                }
                                "CHANNEL_LEFT" => {
                                    Some(OverlayEvent::ChannelLeft)
                                }
                                "VOICE_STATE_UPDATE" => {
                                    serde_json::from_str::<VoiceStateUpdateMessage>(text)
                                        .ok()
                                        .map(|m| OverlayEvent::VoiceStateUpdate(m.state))
                                }
                                "MESSAGE_NOTIFICATION" => {
                                    serde_json::from_str::<MessageNotification>(text)
                                        .ok()
                                        .map(|m| OverlayEvent::MessageNotification(m.message))
                                }
                                _ => {
                                    warn!("Unknown command: {}", generic.cmd);
                                    None
                                }
                            };

                            if let Some(evt) = event {
                                if let Err(e) = tx.send(evt).await {
                                    error!("Failed to send event: {}", e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse message: {} - {}", e, text);
                        }
                    }
                } else if message.is_close() {
                    break;
                }
            }
            Err(e) => {
                warn!("WebSocket error: {}", e);
                break;
            }
        }
    }

    // Send channel left when disconnected
    let _ = tx.send(OverlayEvent::ChannelLeft).await;
    Ok(())
}
