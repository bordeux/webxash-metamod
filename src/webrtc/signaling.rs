//! WebRTC peer connection and signaling.
//!
//! Adapted from the webxash3d-proxy signaling module.

use std::sync::Arc;

use webrtc::api::interceptor_registry::register_default_interceptors;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::api::APIBuilder;
use webrtc::data_channel::data_channel_init::RTCDataChannelInit;
use webrtc::data_channel::RTCDataChannel;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::interceptor::registry::Registry;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::RTCPeerConnection;

/// Create a new WebRTC peer connection with data channels.
///
/// Returns the peer connection and the write/read data channels.
pub async fn create_peer_and_channels(
    public_ip: Option<String>,
) -> Result<
    (
        Arc<RTCPeerConnection>,
        Arc<RTCDataChannel>,
        Arc<RTCDataChannel>,
    ),
    Box<dyn std::error::Error + Send + Sync>,
> {
    let peer = create_peer_connection(public_ip).await?;
    let peer = Arc::new(peer);

    let dc_options = RTCDataChannelInit {
        ordered: Some(true),
        ..Default::default()
    };

    // Create "write" channel - for sending data TO the browser
    let write_channel = peer
        .create_data_channel("write", Some(dc_options.clone()))
        .await?;

    // Create "read" channel - for receiving data FROM the browser
    let read_channel = peer.create_data_channel("read", Some(dc_options)).await?;

    Ok((peer, write_channel, read_channel))
}

/// Create a new WebRTC peer connection.
async fn create_peer_connection(
    public_ip: Option<String>,
) -> Result<RTCPeerConnection, Box<dyn std::error::Error + Send + Sync>> {
    let mut media_engine = MediaEngine::default();
    media_engine.register_default_codecs()?;

    let mut registry = Registry::new();
    registry = register_default_interceptors(registry, &mut media_engine)?;

    let mut setting_engine = SettingEngine::default();

    // Set public IP for NAT traversal if provided
    if let Some(ip) = public_ip {
        setting_engine.set_nat_1to1_ips(
            vec![ip],
            webrtc::ice_transport::ice_candidate_type::RTCIceCandidateType::Host,
        );
    }

    let api = APIBuilder::new()
        .with_media_engine(media_engine)
        .with_interceptor_registry(registry)
        .with_setting_engine(setting_engine)
        .build();

    let config = RTCConfiguration {
        ice_servers: vec![RTCIceServer {
            urls: vec!["stun:stun.l.google.com:19302".to_string()],
            ..Default::default()
        }],
        ..Default::default()
    };

    let peer = api.new_peer_connection(config).await?;

    Ok(peer)
}
