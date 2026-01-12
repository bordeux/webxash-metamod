//! Plugin configuration.
//!
//! Configuration is set via webxash_* cvars.

/// Plugin configuration
#[derive(Debug, Clone)]
pub struct PluginConfig {
    /// HTTP/WebSocket server port (from webxash_port cvar)
    pub http_port: u16,
    /// Game server port (same as http_port by default)
    pub game_port: u16,
    /// Public IP for NAT traversal (from webxash_public_ip cvar)
    pub public_ip: Option<String>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            http_port: 27015,
            game_port: 27015,
            public_ip: None,
        }
    }
}
