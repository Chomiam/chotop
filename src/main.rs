mod avatar_cache;
mod config;
mod control_ipc;
mod control_window;
mod discord_data;
mod ipc;
mod notification_window;
mod overlay_window;
mod renderer;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{glib, Application, ApplicationWindow};
use tokio::sync::mpsc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use avatar_cache::AvatarCache;
use config::Config;
use control_ipc::{ControlCommand, ControlIpcServer};
use discord_data::OverlayEvent;
use ipc::WebSocketServer;
use notification_window::NotificationWindow;
use overlay_window::{load_css, setup_layer_shell};
use renderer::{AvatarRequest, OverlayRenderer};

const APP_ID: &str = "com.discord.overlay";
const WEBSOCKET_PORT: u16 = 6888; // Orbolay default port

fn main() -> glib::ExitCode {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    info!("Starting Discord Overlay Daemon (Orbolay compatible)");
    info!("Listening on port {}", WEBSOCKET_PORT);

    // Create GTK application (non-unique to allow multiple instances during dev)
    let app = Application::builder()
        .application_id(APP_ID)
        .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(build_ui);

    app.run()
}

fn build_ui(app: &Application) {
    // Load config
    let config = Config::load();

    // Load CSS styles
    load_css(&config);

    // Create the main window
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Discord Overlay")
        .decorated(false)
        .resizable(false)
        .build();

    // Setup layer-shell for Wayland overlay
    setup_layer_shell(&window, &config);

    // Create renderer (voice overlay)
    let renderer = Rc::new(RefCell::new(OverlayRenderer::new()));
    window.set_child(Some(renderer.borrow().widget()));

    // Create notification window (separate window for messages)
    let notification_window = Rc::new(RefCell::new(NotificationWindow::new(app, &config)));

    // Create channel for overlay events
    let (event_tx, mut event_rx) = mpsc::channel::<OverlayEvent>(100);

    // Create channel for avatar requests
    let (avatar_tx, mut avatar_rx) = mpsc::channel::<AvatarRequest>(100);

    // Create channel for avatar responses (user_id, path)
    let (avatar_done_tx, mut avatar_done_rx) = mpsc::channel::<(String, PathBuf)>(100);

    // Create channel for control commands
    let (control_tx, mut control_rx) = mpsc::channel::<ControlCommand>(100);

    // Set avatar sender in renderer
    renderer.borrow_mut().set_avatar_sender(avatar_tx);

    // Spawn WebSocket server in tokio runtime
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            let server = WebSocketServer::new(WEBSOCKET_PORT);
            server.run(event_tx).await;
        });
    });

    // Spawn avatar download handler
    let avatar_done_tx_clone = avatar_done_tx.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            let cache = AvatarCache::new();

            while let Some(request) = avatar_rx.recv().await {
                if let Some(path) = cache.get_avatar(&request.user_id, &request.avatar_hash).await {
                    let _ = avatar_done_tx_clone.send((request.user_id, path)).await;
                }
            }
        });
    });

    // Spawn IPC control server
    let control_tx_clone = control_tx.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            use tokio::io::AsyncReadExt;
            use tokio::net::UnixListener;

            let socket_path = ControlIpcServer::get_socket_path();

            // Remove old socket if exists
            if socket_path.exists() {
                let _ = std::fs::remove_file(&socket_path);
            }

            match UnixListener::bind(&socket_path) {
                Ok(listener) => {
                    info!("IPC control server listening on {:?}", socket_path);

                    loop {
                        match listener.accept().await {
                            Ok((mut stream, _)) => {
                                let mut buffer = vec![0u8; 4096];

                                match stream.read(&mut buffer).await {
                                    Ok(n) if n > 0 => {
                                        match serde_json::from_slice::<ControlCommand>(&buffer[..n]) {
                                            Ok(command) => {
                                                info!("Received IPC command: {:?}", command);
                                                let _ = control_tx_clone.send(command).await;
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to deserialize command: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Failed to read from socket: {}", e);
                                    }
                                    _ => {}
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to accept connection: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to bind IPC socket: {}", e);
                }
            }
        });
    });

    // Setup GTK main context to receive overlay events
    let renderer_clone = renderer.clone();
    let notification_window_clone = notification_window.clone();
    glib::spawn_future_local(async move {
        while let Some(event) = event_rx.recv().await {
            match event {
                OverlayEvent::ChannelJoined(users, channel_name) => {
                    info!("Channel joined: {} with {} users", channel_name, users.len());
                    renderer_clone.borrow_mut().on_channel_joined(users, channel_name);
                }
                OverlayEvent::ChannelLeft => {
                    info!("Channel left");
                    renderer_clone.borrow_mut().on_channel_left();
                }
                OverlayEvent::VoiceStateUpdate(update) => {
                    renderer_clone.borrow_mut().on_voice_state_update(update);
                }
                OverlayEvent::ConfigReceived(config) => {
                    info!("Config received from user: {:?}", config.user_id);
                }
                OverlayEvent::MessageNotification(notif) => {
                    info!("Message notification: {}", notif.title);
                    notification_window_clone.borrow_mut().show_notification(notif);
                }
            }
        }
    });

    // Setup GTK main context to receive avatar updates
    let renderer_clone2 = renderer.clone();
    glib::spawn_future_local(async move {
        while let Some((user_id, path)) = avatar_done_rx.recv().await {
            renderer_clone2.borrow_mut().set_avatar(&user_id, &path);
        }
    });

    // Setup GTK main context to receive control commands
    let renderer_clone3 = renderer.clone();
    glib::spawn_future_local(async move {
        while let Some(command) = control_rx.recv().await {
            match command {
                ControlCommand::EnableTestMode => {
                    info!("Enabling test mode");
                    renderer_clone3.borrow_mut().enable_test_mode();
                }
                ControlCommand::DisableTestMode => {
                    info!("Disabling test mode");
                    renderer_clone3.borrow_mut().disable_test_mode();
                }
                ControlCommand::UpdateConfig(_config) => {
                    info!("Config update received (not yet implemented)");
                    // TODO: Reload CSS and reposition windows
                }
                ControlCommand::Restart => {
                    info!("Restart requested (not yet implemented)");
                    // TODO: Implement daemon restart
                }
                ControlCommand::Quit => {
                    info!("Quit requested");
                    std::process::exit(0);
                }
            }
        }
    });

    window.present();
    notification_window.borrow().present();
    info!("Overlay windows created and displayed");
}
