//! HTTP server implementation.
//!
//! Handles both HTTP requests and WebSocket connections.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

use crate::config::PluginConfig;
use crate::server::websocket::handle_websocket;

/// Allowed asset folders for static file serving
const ALLOWED_FOLDERS: &[&str] = &["sound", "sprites", "gfx", "maps", "models", "overviews"];

/// HTTP/WebSocket server
pub struct Server {
    config: Arc<PluginConfig>,
}

impl Server {
    /// Create a new server instance.
    pub fn new(config: PluginConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Run the server.
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.http_port));
        let listener = TcpListener::bind(addr).await?;

        println!("[WEBXASH] HTTP server listening on {addr}");

        loop {
            let (stream, peer_addr) = match listener.accept().await {
                Ok(conn) => conn,
                Err(e) => {
                    eprintln!("[WEBXASH] Accept error: {e}");
                    continue;
                }
            };

            let config = self.config.clone();

            tokio::spawn(async move {
                if let Err(e) = handle_connection(stream, config, peer_addr).await {
                    // Ignore normal connection close errors
                    let err_str = e.to_string();
                    if !err_str.contains("connection closed")
                        && !err_str.contains("Connection reset")
                    {
                        eprintln!("[WEBXASH] Connection error: {e}");
                    }
                }
            });
        }
    }
}

/// Handle a TCP connection - determine if it's WebSocket or HTTP.
async fn handle_connection(
    stream: TcpStream,
    config: Arc<PluginConfig>,
    peer_addr: SocketAddr,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Peek at the first bytes to determine request type
    let mut buf_reader = BufReader::new(stream);
    let mut first_line = String::new();

    // Read the first line to check the request
    buf_reader.read_line(&mut first_line).await?;

    // Check if this looks like a WebSocket upgrade request
    let is_websocket = first_line.contains("/ws") || first_line.contains("/websocket");

    // Read remaining headers
    let mut headers = String::new();
    let mut upgrade_header = false;
    let mut ws_key = String::new();

    loop {
        let mut line = String::new();
        let n = buf_reader.read_line(&mut line).await?;
        if n == 0 || line == "\r\n" || line == "\n" {
            break;
        }

        let line_lower = line.to_lowercase();
        if line_lower.starts_with("upgrade:") && line_lower.contains("websocket") {
            upgrade_header = true;
        }
        if line_lower.starts_with("sec-websocket-key:") {
            ws_key = line
                .split(':')
                .nth(1)
                .unwrap_or("")
                .trim()
                .to_string();
        }

        headers.push_str(&line);
    }

    // Get the underlying stream back
    let stream = buf_reader.into_inner();

    if is_websocket && upgrade_header && !ws_key.is_empty() {
        // Handle WebSocket connection
        let client_id = format!("{}-{}", peer_addr, uuid_simple());
        handle_websocket(stream, config, client_id).await;
    } else {
        // Handle HTTP request
        handle_http_request(stream, &first_line, &config).await?;
    }

    Ok(())
}

/// CORS headers for cross-origin requests
const CORS_HEADERS: &str = "\
Access-Control-Allow-Origin: *\r\n\
Access-Control-Allow-Methods: *\r\n\
Access-Control-Allow-Headers: *\r\n\
Access-Control-Max-Age: 86400";

/// Handle a plain HTTP request.
async fn handle_http_request(
    mut stream: TcpStream,
    first_line: &str,
    _config: &PluginConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    let method = parts.first().unwrap_or(&"GET");
    let path = parts.get(1).unwrap_or(&"/");

    // Handle CORS preflight
    if *method == "OPTIONS" {
        let response = format!(
            "HTTP/1.1 204 No Content\r\n{CORS_HEADERS}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        );
        stream.write_all(response.as_bytes()).await?;
        return Ok(());
    }

    // Handle static file serving for /cstrike/*
    if *method == "GET" && path.starts_with("/cstrike/") {
        return serve_static_file(&mut stream, path).await;
    }

    let (status, content_type, body) = match (*method, *path) {
        ("GET", "/health") => ("200 OK", "text/plain", "OK".to_string()),
        _ => ("404 Not Found", "text/plain", "Not Found".to_string()),
    };

    let response = format!(
        "HTTP/1.1 {status}\r\n{CORS_HEADERS}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );

    stream.write_all(response.as_bytes()).await?;

    Ok(())
}

/// Serve static files from cstrike folder.
async fn serve_static_file(
    stream: &mut TcpStream,
    url_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Remove /cstrike/ prefix to get relative path
    let relative_path = url_path.strip_prefix("/cstrike/").unwrap_or("");

    // URL decode the path
    let relative_path = url_decode(relative_path);

    // Security: check for path traversal
    if relative_path.contains("..") || relative_path.starts_with('/') {
        send_error(stream, "403 Forbidden", "Access denied").await?;
        return Ok(());
    }

    // Check if the first folder is in the allowed list
    let first_folder = relative_path.split('/').next().unwrap_or("");
    if !ALLOWED_FOLDERS.contains(&first_folder) {
        send_error(stream, "403 Forbidden", "Folder not allowed").await?;
        return Ok(());
    }

    // Build the file path (relative to HLDS working directory)
    let file_path = Path::new("cstrike").join(&relative_path);

    // Try to open and read the file
    let file = match File::open(&file_path).await {
        Ok(f) => f,
        Err(_) => {
            send_error(stream, "404 Not Found", "File not found").await?;
            return Ok(());
        }
    };

    // Get file size
    let metadata = match file.metadata().await {
        Ok(m) => m,
        Err(_) => {
            send_error(stream, "500 Internal Server Error", "Cannot read file").await?;
            return Ok(());
        }
    };

    let content_type = get_content_type(&relative_path);
    let content_length = metadata.len();

    // Send headers
    let headers = format!(
        "HTTP/1.1 200 OK\r\n{CORS_HEADERS}\r\nContent-Type: {content_type}\r\nContent-Length: {content_length}\r\nConnection: close\r\n\r\n"
    );
    stream.write_all(headers.as_bytes()).await?;

    // Stream file contents
    let mut file = file;
    let mut buffer = vec![0u8; 65536];
    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        stream.write_all(&buffer[..n]).await?;
    }

    Ok(())
}

/// Send an error response.
async fn send_error(
    stream: &mut TcpStream,
    status: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = format!(
        "HTTP/1.1 {status}\r\n{CORS_HEADERS}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{message}",
        message.len()
    );
    stream.write_all(response.as_bytes()).await?;
    Ok(())
}

/// Get content type based on file extension.
fn get_content_type(path: &str) -> &'static str {
    let ext = path.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        // Audio
        "wav" => "audio/wav",
        "mp3" => "audio/mpeg",
        // Images
        "bmp" => "image/bmp",
        "tga" => "image/x-tga",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        // Models/sprites
        "mdl" => "application/octet-stream",
        "spr" => "application/octet-stream",
        // Maps
        "bsp" => "application/octet-stream",
        "wad" => "application/octet-stream",
        "res" => "text/plain",
        "txt" => "text/plain",
        // Default
        _ => "application/octet-stream",
    }
}

/// Simple URL decode (handles %XX sequences).
fn url_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            let hex: String = chars.by_ref().take(2).collect();
            if hex.len() == 2 {
                if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                    result.push(byte as char);
                    continue;
                }
            }
            result.push('%');
            result.push_str(&hex);
        } else if c == '+' {
            result.push(' ');
        } else {
            result.push(c);
        }
    }

    result
}

/// Generate a simple UUID-like string.
fn uuid_simple() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    format!("{:x}{:x}", now.as_secs(), now.subsec_nanos())
}
