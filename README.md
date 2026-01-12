# webxash-metamod

A Rust-based Metamod plugin that provides WebRTC-to-UDP bridging for browser-based Half-Life/CS 1.6 clients. This plugin embeds a WebRTC signaling server directly into the HLDS game server, allowing browser clients to connect via WebRTC data channels.

## Features

- **Embedded WebRTC Server** - Runs inside HLDS as a Metamod plugin
- **WebSocket Signaling** - Built-in signaling server for WebRTC negotiation
- **UDP Bridge** - Bidirectional packet forwarding between WebRTC and game server
- **Cross-Platform** - Supports Linux and Windows (32-bit targets for HLDS compatibility)

## Architecture

```
┌──────────────────────────────────────────────────────────────────────────┐
│  HLDS Process                                                            │
│                                                                          │
│  ┌────────────────────┐     ┌────────────────────────────────────────┐  │
│  │  Game Engine       │     │  Metamod Plugin (webxash-metamod)    │  │
│  │                    │     │                                        │  │
│  │  UDP :27015        │◄───►│  ┌──────────────────────────────────┐  │  │
│  │  (Game traffic)    │     │  │  Bridge (per WebRTC client)      │  │  │
│  │                    │     │  │  UDP 127.0.0.1:ephemeral         │  │  │
│  └────────────────────┘     │  │        ▲                         │  │  │
│                             │  │        │                         │  │  │
│                             │  │        ▼                         │  │  │
│                             │  │  WebRTC Data Channels            │  │  │
│                             │  │  (read/write)                    │  │  │
│                             │  └──────────────────────────────────┘  │  │
│                             │                   ▲                    │  │
│                             │                   │                    │  │
│                             │  ┌────────────────┴───────────────┐   │  │
│                             │  │  HTTP/WebSocket Server         │   │  │
│                             │  │  (TCP on same port as game)    │   │  │
│                             │  └────────────────────────────────┘   │  │
│                             └────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
                                          │
                                          │ WebSocket + WebRTC
                                          ▼
                              ┌─────────────────────────┐
                              │  Browser Client         │
                              │  (Xash3D WASM)          │
                              └─────────────────────────┘
```

## Building

### Prerequisites

- Rust 1.70+ with Cargo
- For cross-compilation to 32-bit targets:
  - Linux: `rustup target add i686-unknown-linux-gnu` and 32-bit libc
  - Windows: `rustup target add i686-pc-windows-gnu` and MinGW-w64

### Build Commands

```bash
# Development build (native target)
cargo build

# Release build (native target)
cargo build --release

# Linux 32-bit (for HLDS)
cargo build --release --target i686-unknown-linux-gnu

# Windows 32-bit (for HLDS)
cargo build --release --target i686-pc-windows-gnu
```

### Output Files

| Target | Output Path |
|--------|-------------|
| macOS (dev) | `target/release/libwebxash_metamod.dylib` |
| Linux 32-bit | `target/i686-unknown-linux-gnu/release/libwebxash_metamod.so` |
| Windows 32-bit | `target/i686-pc-windows-gnu/release/webxash_metamod.dll` |

## Installation

1. Build the plugin for your target platform
2. Copy the output library to your HLDS `addons/webxash/` directory
3. Add to `addons/metamod/plugins.ini`:
   ```
   linux addons/webxash/libwebxash_metamod.so
   win32 addons/webxash/webxash_metamod.dll
   ```
4. Restart HLDS - plugin starts HTTP server on first map load

## Configuration

The plugin automatically reads the HLDS `hostport` cvar to determine which port to use for the HTTP/WebSocket server. Since HLDS uses UDP and the plugin uses TCP, they can share the same port number.

No additional configuration is required - the plugin auto-detects the server port on map load.

## API Endpoints

### GET /health
Health check endpoint. Returns `OK` if the server is running.

### WebSocket /ws or /websocket
WebRTC signaling endpoint. Accepts WebSocket connections for SDP offer/answer and ICE candidate exchange.

### GET /cstrike/*
Static file server for game assets. Serves files from the HLDS `cstrike/` directory with path traversal protection.

**Allowed folders:** `sound`, `sprites`, `gfx`, `maps`, `models`, `overviews`

**Example:** `/cstrike/sound/weapons/ak47-1.wav`

## WebSocket Signaling Protocol

Messages are JSON objects with `event` and `data` fields:

```json
// Server -> Client: Offer
{"event": "offer", "data": {"type": "offer", "sdp": "..."}}

// Client -> Server: Answer
{"event": "answer", "data": {"type": "answer", "sdp": "..."}}

// Bidirectional: ICE Candidate
{"event": "candidate", "data": {"candidate": "...", "sdpMid": "...", "sdpMLineIndex": 0}}
```

## Project Structure

```
src/
├── lib.rs              # Library entry point
├── plugin.rs           # Plugin state and lifecycle
├── metamod/
│   ├── mod.rs
│   ├── types.rs        # Metamod SDK FFI types
│   └── exports.rs      # Metamod API exports (Meta_Init, etc.)
├── runtime/
│   └── mod.rs          # Tokio runtime management
├── server/
│   ├── mod.rs
│   ├── http.rs         # HTTP server
│   └── websocket.rs    # WebSocket signaling handler
├── webrtc/
│   ├── mod.rs
│   └── signaling.rs    # WebRTC peer connection setup
├── bridge/
│   └── mod.rs          # UDP <-> WebRTC packet bridge
└── config/
    └── mod.rs          # Plugin configuration
```

## Requirements

### Game Server
- HLDS (Half-Life Dedicated Server)
- Metamod or Metamod-P installed
- ReUnion module (for non-Steam client support)

### Browser Client
- WebRTC-capable browser
- Xash3D WASM client (from webxash3d-proxy project)

## Related Projects

- [webxash3d-proxy](../webxash3d-proxy) - Standalone WebRTC proxy server with web client
- [xash3d-fwgs](https://github.com/nicexash3d/nicexash3d-nicewasm) - Xash3D WASM engine

## License

MIT
