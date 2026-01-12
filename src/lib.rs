//! WebXash3D Metamod Plugin
//!
//! A Metamod plugin that provides WebRTC-to-UDP bridging for browser-based
//! Half-Life/CS 1.6 clients.
//!
//! This plugin embeds a WebRTC signaling server directly into the HLDS game
//! server, allowing browser clients to connect via WebRTC data channels.

#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

mod bridge;
mod config;
mod metamod;
mod plugin;
mod runtime;
mod server;
mod webrtc;

// Re-export the Metamod exports for the DLL
pub use metamod::exports::*;
