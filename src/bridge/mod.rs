//! UDP bridge between WebRTC data channels and game server.
//!
//! Adapted from the webxash3d-proxy bridge module.

use std::sync::Arc;

use tokio::net::UdpSocket;
use tokio::sync::Notify;
use webrtc::data_channel::data_channel_message::DataChannelMessage;
use webrtc::data_channel::RTCDataChannel;

/// Maximum packet size for GoldSrc protocol
const MAX_PACKET_SIZE: usize = 65536;

/// Bridge between WebRTC data channels and UDP socket to game server.
///
/// Uses two channels to match the original client expectations:
/// - `write` channel: proxy sends TO browser (server -> client)
/// - `read` channel: proxy receives FROM browser (client -> server)
pub struct Bridge {
    /// Channel for sending data TO the browser (server responses)
    write_channel: Arc<RTCDataChannel>,
    /// Channel for receiving data FROM the browser (client commands)
    read_channel: Arc<RTCDataChannel>,
    /// UDP socket connected to game server
    udp_socket: Arc<UdpSocket>,
    /// Shutdown signal
    shutdown: Arc<Notify>,
    /// Client identifier for logging
    client_id: String,
}

impl Bridge {
    /// Create a new bridge connecting data channels to a game server.
    pub async fn new(
        write_channel: Arc<RTCDataChannel>,
        read_channel: Arc<RTCDataChannel>,
        server_addr: &str,
        client_id: String,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Bind to random local port on localhost
        let udp_socket = UdpSocket::bind("127.0.0.1:0").await?;

        // Connect to game server (allows us to use send/recv instead of send_to/recv_from)
        udp_socket.connect(server_addr).await?;

        let local_addr = udp_socket.local_addr()?;
        println!(
            "[WEBXASH] Bridge {}: UDP socket {} -> {}",
            client_id, local_addr, server_addr
        );

        Ok(Self {
            write_channel,
            read_channel,
            udp_socket: Arc::new(udp_socket),
            shutdown: Arc::new(Notify::new()),
            client_id,
        })
    }

    /// Start bidirectional forwarding.
    pub async fn start(self: Arc<Self>) {
        // Spawn UDP -> WebRTC forwarder (server responses to browser via write channel)
        let udp_to_webrtc = tokio::spawn({
            let bridge = self.clone();
            async move {
                bridge.forward_udp_to_webrtc().await;
            }
        });

        // Setup WebRTC -> UDP forwarder (browser commands to server via read channel)
        self.setup_webrtc_to_udp();

        // Wait for shutdown signal
        self.shutdown.notified().await;

        // Cleanup
        udp_to_webrtc.abort();
        println!("[WEBXASH] Bridge {} shut down", self.client_id);
    }

    /// Forward packets from UDP (game server) to WebRTC write channel (browser).
    async fn forward_udp_to_webrtc(&self) {
        let mut buf = vec![0u8; MAX_PACKET_SIZE];

        loop {
            tokio::select! {
                result = self.udp_socket.recv(&mut buf) => {
                    match result {
                        Ok(n) if n > 0 => {
                            let data = bytes::Bytes::copy_from_slice(&buf[..n]);

                            if let Err(e) = self.write_channel.send(&data).await {
                                eprintln!(
                                    "[WEBXASH] Bridge {}: Failed to send to write channel: {}",
                                    self.client_id, e
                                );
                                break;
                            }
                        }
                        Ok(_) => {
                            // Empty packet, continue
                        }
                        Err(e) => {
                            eprintln!(
                                "[WEBXASH] Bridge {}: UDP recv error: {}",
                                self.client_id, e
                            );
                            break;
                        }
                    }
                }
                () = self.shutdown.notified() => {
                    break;
                }
            }
        }
    }

    /// Setup callback for WebRTC read channel -> UDP forwarding (browser to game server).
    fn setup_webrtc_to_udp(&self) {
        let udp_socket = self.udp_socket.clone();
        let client_id = self.client_id.clone();
        let shutdown = self.shutdown.clone();

        // Handle incoming messages on the read channel
        self.read_channel
            .on_message(Box::new(move |msg: DataChannelMessage| {
                let udp_socket = udp_socket.clone();
                let client_id = client_id.clone();

                Box::pin(async move {
                    let data = msg.data;

                    if let Err(e) = udp_socket.send(&data).await {
                        eprintln!(
                            "[WEBXASH] Bridge {}: Failed to send to UDP: {}",
                            client_id, e
                        );
                    }
                })
            }));

        // Handle read channel close
        let shutdown_clone = shutdown.clone();
        let client_id = self.client_id.clone();
        self.read_channel.on_close(Box::new(move || {
            println!("[WEBXASH] Bridge {}: Read channel closed", client_id);
            shutdown_clone.notify_one();
            Box::pin(async {})
        }));

        // Handle read channel errors
        let shutdown_clone = shutdown;
        let client_id = self.client_id.clone();
        self.read_channel.on_error(Box::new(move |e| {
            eprintln!("[WEBXASH] Bridge {}: Read channel error: {}", client_id, e);
            shutdown_clone.notify_one();
            Box::pin(async {})
        }));
    }

    /// Shutdown the bridge.
    pub fn shutdown(&self) {
        self.shutdown.notify_one();
    }
}

impl Drop for Bridge {
    fn drop(&mut self) {
        println!("[WEBXASH] Bridge {} dropped", self.client_id);
    }
}
