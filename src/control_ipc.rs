use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{UnixListener, UnixStream};
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ControlCommand {
    EnableTestMode,
    DisableTestMode,
    UpdateConfig(crate::config::Config),
    Restart,
    Quit,
}

pub struct ControlIpcServer {
    socket_path: PathBuf,
}

impl ControlIpcServer {
    pub fn new() -> Self {
        let socket_path = Self::get_socket_path();

        // Remove old socket if it exists
        if socket_path.exists() {
            let _ = std::fs::remove_file(&socket_path);
        }

        Self { socket_path }
    }

    pub fn get_socket_path() -> PathBuf {
        let runtime_dir = std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(runtime_dir).join("chotop-control.sock")
    }

    pub async fn run(
        &self,
        mut command_rx: tokio::sync::mpsc::Receiver<ControlCommand>,
    ) -> std::io::Result<()> {
        let listener = UnixListener::bind(&self.socket_path)?;
        info!("Control IPC server listening on {:?}", self.socket_path);

        loop {
            tokio::select! {
                Ok((stream, _)) = listener.accept() => {
                    self.handle_connection(stream).await;
                }
                Some(cmd) = command_rx.recv() => {
                    // Handle internal commands if needed
                    info!("Received internal command: {:?}", cmd);
                }
            }
        }
    }

    async fn handle_connection(&self, mut stream: UnixStream) {
        let mut buffer = vec![0u8; 4096];

        match stream.read(&mut buffer).await {
            Ok(n) if n > 0 => {
                match serde_json::from_slice::<ControlCommand>(&buffer[..n]) {
                    Ok(command) => {
                        info!("Received command: {:?}", command);

                        // Send acknowledgment
                        let response = b"OK";
                        let _ = stream.write_all(response).await;
                    }
                    Err(e) => {
                        error!("Failed to deserialize command: {}", e);
                    }
                }
            }
            Ok(_) => {
                info!("Empty message received");
            }
            Err(e) => {
                error!("Failed to read from socket: {}", e);
            }
        }
    }
}

impl Drop for ControlIpcServer {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

pub struct ControlIpcClient;

impl ControlIpcClient {
    pub async fn send_command(command: ControlCommand) -> std::io::Result<()> {
        let socket_path = ControlIpcServer::get_socket_path();

        let mut stream = UnixStream::connect(&socket_path).await?;

        let data = serde_json::to_vec(&command)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        stream.write_all(&data).await?;

        // Wait for acknowledgment
        let mut response = vec![0u8; 1024];
        let n = stream.read(&mut response).await?;

        if &response[..n] == b"OK" {
            info!("Command sent successfully");
        }

        Ok(())
    }
}
