//! WebSocket signaling handler.

use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::{protocol::Role, Message},
    WebSocketStream,
};

use crate::bridge::Bridge;
use crate::config::PluginConfig;
use crate::webrtc::create_peer_and_channels;

/// Signal event types
mod events {
    pub const OFFER: &str = "offer";
    pub const ANSWER: &str = "answer";
    pub const CANDIDATE: &str = "candidate";
}

/// WebSocket signaling message
#[derive(Debug, Serialize, Deserialize)]
struct SignalMessage {
    event: String,
    data: serde_json::Value,
}

/// Handle a WebSocket connection for signaling.
///
/// Note: The TCP stream has already completed the WebSocket handshake in http.rs.
/// We wrap it directly as a WebSocketStream since the HTTP 101 response was already sent.
pub async fn handle_websocket(stream: TcpStream, config: Arc<PluginConfig>, client_id: String) {
    println!("[WEBXASH] New WebSocket connection: {client_id}");

    // Wrap the stream as WebSocket (handshake already completed in http.rs)
    let ws_stream = WebSocketStream::from_raw_socket(stream, Role::Server, None).await;

    // Handle the signaling
    if let Err(e) = handle_signaling(ws_stream, config, client_id.clone()).await {
        eprintln!("[WEBXASH] Signaling error for {client_id}: {e}");
    }

    println!("[WEBXASH] WebSocket connection closed: {client_id}");
}

/// Handle WebRTC signaling over WebSocket.
async fn handle_signaling(
    ws_stream: WebSocketStream<TcpStream>,
    config: Arc<PluginConfig>,
    client_id: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (mut ws_sender, mut ws_receiver) = ws_stream.split();

    // Create peer connection and data channels
    let (peer, write_channel, read_channel) =
        create_peer_and_channels(config.public_ip.clone()).await?;

    println!("[WEBXASH] Created peer connection for {client_id}");

    // Create the offer
    let offer = peer.create_offer(None).await?;
    peer.set_local_description(offer.clone()).await?;

    // Send offer to client
    let offer_msg = SignalMessage {
        event: events::OFFER.to_string(),
        data: serde_json::json!({
            "type": events::OFFER,
            "sdp": offer.sdp
        }),
    };

    let json = serde_json::to_string(&offer_msg)?;
    ws_sender.send(Message::Text(json)).await?;

    println!("[WEBXASH] Sent offer to {client_id}");

    // Set up ICE candidate handler
    let ws_sender_arc = Arc::new(tokio::sync::Mutex::new(ws_sender));
    let ws_sender_for_ice = ws_sender_arc.clone();
    let client_id_for_ice = client_id.clone();

    peer.on_ice_candidate(Box::new(move |candidate| {
        let ws_sender = ws_sender_for_ice.clone();
        let client_id = client_id_for_ice.clone();

        Box::pin(async move {
            let Some(c) = candidate else {
                return;
            };

            match c.to_json() {
                Ok(json) => {
                    let msg = SignalMessage {
                        event: events::CANDIDATE.to_string(),
                        data: serde_json::to_value(json).unwrap_or_default(),
                    };

                    let json_str = serde_json::to_string(&msg).unwrap_or_default();
                    let mut sender = ws_sender.lock().await;
                    if let Err(e) = sender.send(Message::Text(json_str)).await {
                        eprintln!("[WEBXASH] Failed to send ICE candidate to {client_id}: {e}");
                    }
                }
                Err(e) => {
                    eprintln!("[WEBXASH] Failed to serialize ICE candidate: {e}");
                }
            }
        })
    }));

    // Set up bridge when channels are ready
    let bridge = Arc::new(tokio::sync::Mutex::new(Option::<Arc<Bridge>>::None));
    let bridge_for_callback = bridge.clone();
    let config_for_bridge = config.clone();
    let client_id_for_bridge = client_id.clone();
    let write_channel_for_bridge = write_channel.clone();
    let read_channel_for_bridge = read_channel.clone();

    // Track channel open states
    let channels_open = Arc::new(std::sync::atomic::AtomicU8::new(0));
    let channels_open_write = channels_open.clone();
    let channels_open_read = channels_open.clone();

    // Clone for both callbacks
    let bridge_for_write = bridge_for_callback.clone();
    let config_for_write = config_for_bridge.clone();
    let client_id_for_write = client_id_for_bridge.clone();
    let write_channel_for_write = write_channel_for_bridge.clone();
    let read_channel_for_write = read_channel_for_bridge.clone();

    write_channel.on_open(Box::new(move || {
        let channels_open = channels_open_write.clone();
        let bridge = bridge_for_write.clone();
        let config = config_for_write.clone();
        let client_id = client_id_for_write.clone();
        let write_channel = write_channel_for_write.clone();
        let read_channel = read_channel_for_write.clone();

        Box::pin(async move {
            let count = channels_open.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            if count == 2 {
                start_bridge(bridge, config, client_id, write_channel, read_channel).await;
            }
        })
    }));

    let bridge_for_read = bridge_for_callback;
    let config_for_read = config_for_bridge;
    let client_id_for_read = client_id_for_bridge;
    let write_channel_for_read = write_channel_for_bridge;
    let read_channel_for_read = read_channel_for_bridge;

    read_channel.on_open(Box::new(move || {
        let channels_open = channels_open_read.clone();
        let bridge = bridge_for_read.clone();
        let config = config_for_read.clone();
        let client_id = client_id_for_read.clone();
        let write_channel = write_channel_for_read.clone();
        let read_channel = read_channel_for_read.clone();

        Box::pin(async move {
            let count = channels_open.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            if count == 2 {
                start_bridge(bridge, config, client_id, write_channel, read_channel).await;
            }
        })
    }));

    // Handle incoming WebSocket messages
    while let Some(msg) = ws_receiver.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                let signal: SignalMessage = match serde_json::from_str(&text) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("[WEBXASH] Invalid signal message from {client_id}: {e}");
                        continue;
                    }
                };

                match signal.event.as_str() {
                    events::ANSWER => {
                        let sdp = signal
                            .data
                            .get("sdp")
                            .and_then(|s| s.as_str())
                            .unwrap_or("");

                        let answer =
                            webrtc::peer_connection::sdp::session_description::RTCSessionDescription::answer(
                                sdp.to_string(),
                            )?;

                        peer.set_remote_description(answer).await?;
                        println!("[WEBXASH] Set remote description for {client_id}");
                    }
                    events::CANDIDATE => {
                        let candidate: webrtc::ice_transport::ice_candidate::RTCIceCandidateInit =
                            match serde_json::from_value(signal.data) {
                                Ok(c) => c,
                                Err(e) => {
                                    eprintln!(
                                        "[WEBXASH] Invalid ICE candidate from {client_id}: {e}"
                                    );
                                    continue;
                                }
                            };

                        peer.add_ice_candidate(candidate).await?;
                    }
                    _ => {
                        eprintln!(
                            "[WEBXASH] Unknown signal event from {client_id}: {}",
                            signal.event
                        );
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("[WEBXASH] WebSocket close from {client_id}");
                break;
            }
            Ok(Message::Ping(_)) => {
                // Pong is handled automatically
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("[WEBXASH] WebSocket error from {client_id}: {e}");
                break;
            }
        }
    }

    // Cleanup
    if let Some(b) = bridge.lock().await.take() {
        b.shutdown();
    }

    Ok(())
}

/// Start the UDP bridge.
async fn start_bridge(
    bridge_holder: Arc<tokio::sync::Mutex<Option<Arc<Bridge>>>>,
    config: Arc<PluginConfig>,
    client_id: String,
    write_channel: Arc<webrtc::data_channel::RTCDataChannel>,
    read_channel: Arc<webrtc::data_channel::RTCDataChannel>,
) {
    println!("[WEBXASH] Both channels open, starting bridge for {client_id}");

    let server_addr = format!("127.0.0.1:{}", config.game_port);

    match Bridge::new(write_channel, read_channel, &server_addr, client_id.clone()).await {
        Ok(b) => {
            let b = Arc::new(b);
            *bridge_holder.lock().await = Some(b.clone());
            tokio::spawn(async move {
                b.start().await;
            });
        }
        Err(e) => {
            eprintln!("[WEBXASH] Failed to create bridge for {client_id}: {e}");
        }
    }
}
