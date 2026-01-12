# CLAUDE.md - AI Assistant Context

## Git Commits

**Do not include Claude/AI information in commits.** Specifically:
- No "Generated with Claude Code" footer
- No "Co-Authored-By: Claude" lines
- No AI-related mentions in commit messages

Use conventional commits format: `feat:`, `fix:`, `docs:`, `refactor:`, `chore:`, etc.

## Project Summary

**webxash3d-metamod** is a Rust-based Metamod plugin that embeds a WebRTC-to-UDP proxy directly into HLDS (Half-Life Dedicated Server). It allows browser clients running Xash3D WASM to connect to real game servers via WebRTC.

## Project Structure

```
webxash3d-metamod/
├── Cargo.toml                 # cdylib crate, cross-platform config
├── .cargo/config.toml         # Cross-compilation targets
├── src/
│   ├── lib.rs                 # Entry point, module exports
│   ├── plugin.rs              # Plugin state, lifecycle, cvar reading
│   ├── config/
│   │   └── mod.rs             # PluginConfig struct
│   ├── metamod/
│   │   ├── mod.rs
│   │   ├── types.rs           # FFI types (EngineFuncs, PluginInfo, etc.)
│   │   └── exports.rs         # #[no_mangle] extern "C" functions
│   ├── runtime/
│   │   └── mod.rs             # Tokio runtime in dedicated thread
│   ├── server/
│   │   ├── mod.rs
│   │   ├── http.rs            # HTTP server, static files, CORS
│   │   └── websocket.rs       # WebSocket signaling handler
│   ├── webrtc/
│   │   ├── mod.rs
│   │   └── signaling.rs       # WebRTC peer connection, data channels
│   └── bridge/
│       └── mod.rs             # UDP <-> WebRTC packet forwarding
└── scripts/
    └── build-linux.sh         # Docker-based cross-compilation
```

## Architecture

```
┌─────────────────────────┐     ┌─────────────────────────────────────────┐
│  Browser                │     │  HLDS + Metamod Plugin                  │
│  (Xash3D WASM)          │     │                                         │
│                         │     │  ┌─────────────────────────────────┐    │
│  WebSocket signaling ───┼────►│  │ HTTP/WS Server (TCP:hostport)   │    │
│                         │     │  └─────────────────────────────────┘    │
│  WebRTC data channels ◄─┼────►│  ┌─────────────────────────────────┐    │
│                         │     │  │ Bridge (per client)             │    │
│                         │     │  │   UDP 127.0.0.1:hostport ◄──────┼───►│ HLDS
└─────────────────────────┘     │  └─────────────────────────────────┘    │
                                └─────────────────────────────────────────┘
```

- Plugin runs HTTP/WebSocket server on same port as HLDS (TCP alongside UDP)
- Reads `hostport` cvar from HLDS to auto-detect port
- Each WebRTC client gets a Bridge with UDP socket to `127.0.0.1:hostport`

## Key Files

### src/metamod/types.rs
FFI types matching HLSDK structures. **Critical:** `EngineFuncs` struct must exactly match `enginefuncs_t` from HLSDK `eiface.h` (158 function pointers in specific order).

### src/metamod/exports.rs
Metamod API exports:
- `PLUGIN_INFO` - Static plugin metadata
- `Meta_Init()` - Early initialization
- `Meta_Query()` - Return plugin info
- `Meta_Attach()` - Attach to Metamod, register hooks
- `Meta_Detach()` - Cleanup on unload
- `GiveFnptrsToDll()` - Receive engine function pointers

### src/plugin.rs
Plugin state and lifecycle:
- `on_server_activate()` - Start HTTP server on map load (hostport cvar available)
- `load_config()` - Read hostport cvar from engine
- `server_print()` - Log to HLDS console

### src/server/http.rs
HTTP server with:
- WebSocket upgrade detection for `/ws` and `/websocket`
- Static file serving at `/cstrike/*` (sound, sprites, gfx, maps, models, overviews)
- CORS headers allowing all origins
- Health check at `/health`

### src/bridge/mod.rs
Per-client UDP bridge:
- Connects to game server at `127.0.0.1:hostport`
- Forwards packets between WebRTC data channels and UDP

## Build Commands

```bash
# Native build (for development)
cargo build --release

# Cross-compile for Linux 32-bit (HLDS target) using Docker
docker run --rm --platform linux/amd64 \
  -v "$PWD":/app -w /app rust:latest \
  bash -c "rustup target add i686-unknown-linux-gnu && \
           apt-get update && apt-get install -y gcc-multilib && \
           cargo build --release --target i686-unknown-linux-gnu"

# Output: target/i686-unknown-linux-gnu/release/libwebxash3d_metamod.so
```

## Deployment

1. Copy `libwebxash3d_metamod.so` to HLDS `addons/webxash3d/` directory
2. Add to `addons/metamod/plugins.ini`:
   ```
   linux addons/webxash3d/libwebxash3d_metamod.so
   ```
3. Restart HLDS - plugin starts HTTP server on map load

## Key APIs

### WebSocket Signaling (`/ws` or `/websocket`)
```json
{"event": "offer", "data": {"type": "offer", "sdp": "..."}}
{"event": "answer", "data": {"type": "answer", "sdp": "..."}}
{"event": "candidate", "data": {"candidate": "...", "sdpMid": "...", "sdpMLineIndex": ...}}
```

### Static Files (`/cstrike/*`)
Serves game assets from HLDS `cstrike/` directory:
- Only allowed folders: `sound`, `sprites`, `gfx`, `maps`, `models`, `overviews`
- Path traversal protection (rejects `..`)
- Proper MIME types for game files

## Common Tasks

### Add new cvar
1. Define cvar name in `plugin.rs` as `const CVAR_NAME: &[u8] = b"cvar_name\0";`
2. Read in `load_config()` using `funcs.pfn_cvar_get_pointer` (see note below)
3. Store in `PluginConfig` struct

**Important:** Use `pfn_cvar_get_pointer` and read `cvar->value` directly. See "EngineFuncs Struct Offset Issue" below.

### Modify HTTP routes
Edit `src/server/http.rs`:
- Add route in `handle_http_request()` match statement
- For new static folders, add to `ALLOWED_FOLDERS`

### Change WebRTC settings
- ICE servers: `src/webrtc/signaling.rs` RTCConfiguration
- Data channel options: `src/webrtc/signaling.rs` RTCDataChannelInit

## Debugging

### HLDS Console
Plugin logs with `[WEBXASH]` prefix:
```
[WEBXASH] Starting WebRTC server on port 27015
[WEBXASH] WebRTC server started successfully
```

### Common Issues

**Segfault on startup:**
- Usually wrong `EngineFuncs` struct layout
- Compare with HLSDK `eiface.h` field order

**Port shows as 27015 (default):**
- Server starts before map load - hostport cvar not yet set
- Plugin now starts on `server_activate` to fix this

**Plugin version shows garbage:**
- Version string not null-terminated
- Use `concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes()`

### EngineFuncs Struct Offset Issue

**Finding:** The `EngineFuncs` struct layout has incorrect offsets for some functions. Specifically:
- `pfn_cvar_get_float` - returns 0 (wrong function being called)
- `pfn_cvar_get_string` - returns wrong value (cvar->string showed "0" when value was 27016)
- `pfn_cvar_get_pointer` - **works correctly**

**Solution:** Always use `pfn_cvar_get_pointer` to get the cvar struct pointer, then read `cvar->value` directly:

```rust
if let Some(cvar_get_pointer) = funcs.pfn_cvar_get_pointer {
    let cvar_ptr = cvar_get_pointer(cvar_name);
    if !cvar_ptr.is_null() {
        let cvar = &*cvar_ptr;
        let value = cvar.value; // This works correctly
    }
}
```

**Root cause:** The `EngineFuncs` struct in `types.rs` likely has missing or misaligned fields before `pfn_cvar_get_float` (position ~58 in the struct). The `pfn_cvar_get_pointer` field (position ~104) happens to be at the correct offset.

## Dependencies

Key crates:
- `tokio` - Async runtime (runs in dedicated thread)
- `webrtc` - WebRTC implementation
- `parking_lot` - Fast mutex for global plugin state
- `once_cell` - Lazy static initialization

## References

- [HLSDK eiface.h](https://github.com/alliedmodders/hlsdk/blob/master/engine/eiface.h) - Engine functions struct
- [Metamod-P](http://metamod-p.sourceforge.net/) - Metamod plugin interface
- [webxash3d-proxy](../webxash3d-proxy/) - Standalone proxy this was derived from
