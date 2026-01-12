//! Tokio runtime management for the plugin.
//!
//! Runs the async runtime in a dedicated thread to avoid conflicts with the game engine.

use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use tokio::runtime::Runtime;
use tokio::sync::mpsc;

use crate::config::PluginConfig;
use crate::server::Server;

/// Commands that can be sent to the runtime
enum RuntimeCommand {
    Shutdown,
}

/// Plugin runtime that manages the async server in a dedicated thread.
pub struct PluginRuntime {
    /// Thread handle for the runtime thread
    thread_handle: Option<JoinHandle<()>>,
    /// Shutdown flag
    shutdown_flag: Arc<AtomicBool>,
    /// Command sender
    command_tx: Option<mpsc::UnboundedSender<RuntimeCommand>>,
}

impl PluginRuntime {
    /// Create a new plugin runtime.
    pub fn new() -> Self {
        Self {
            thread_handle: None,
            shutdown_flag: Arc::new(AtomicBool::new(false)),
            command_tx: None,
        }
    }

    /// Start the runtime and server.
    pub fn start(
        &mut self,
        config: PluginConfig,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let shutdown_flag = self.shutdown_flag.clone();
        let (tx, mut rx) = mpsc::unbounded_channel::<RuntimeCommand>();
        self.command_tx = Some(tx);

        let handle = thread::Builder::new()
            .name("webrtc-runtime".to_string())
            .spawn(move || {
                // Set up panic hook to prevent crashes from propagating to HLDS
                let default_hook = panic::take_hook();
                panic::set_hook(Box::new(move |info| {
                    eprintln!("[WEBXASH] Panic in async runtime: {info}");
                    // Don't call default hook - we don't want to abort
                }));

                // Wrap everything in catch_unwind for extra safety
                let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                    run_server(config, shutdown_flag, rx);
                }));

                if let Err(e) = result {
                    eprintln!("[WEBXASH] Runtime thread panicked: {e:?}");
                }

                // Restore default panic hook
                panic::set_hook(default_hook);
            })?;

        self.thread_handle = Some(handle);
        Ok(())
    }

    /// Shutdown the runtime.
    pub fn shutdown(&mut self) {
        // Set the shutdown flag
        self.shutdown_flag.store(true, Ordering::SeqCst);

        // Send shutdown command
        if let Some(tx) = self.command_tx.take() {
            let _ = tx.send(RuntimeCommand::Shutdown);
        }

        // Wait for thread to finish with timeout
        if let Some(handle) = self.thread_handle.take() {
            // Give it some time to shutdown gracefully
            let _ = handle.join();
        }
    }
}

/// Run the server in the tokio runtime.
fn run_server(
    config: PluginConfig,
    shutdown_flag: Arc<AtomicBool>,
    mut rx: mpsc::UnboundedReceiver<RuntimeCommand>,
) {
    // Create a multi-threaded runtime
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("[WEBXASH] Failed to create Tokio runtime: {e}");
            return;
        }
    };

    rt.block_on(async move {
        // Create and run the server
        let server = Server::new(config);

        tokio::select! {
            biased;

            // Check shutdown flag first
            _ = async {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    if shutdown_flag.load(Ordering::SeqCst) {
                        break;
                    }
                }
            } => {
                println!("[WEBXASH] Shutdown flag detected");
            }

            // Check for shutdown command
            _ = async {
                // Wait for any command (currently only Shutdown exists)
                if let Some(RuntimeCommand::Shutdown) = rx.recv().await {
                    // Shutdown received
                }
            } => {
                println!("[WEBXASH] Shutdown command received");
            }

            // Run the server
            result = server.run() => {
                if let Err(e) = result {
                    eprintln!("[WEBXASH] Server error: {e}");
                }
            }
        }
    });

    println!("[WEBXASH] Runtime shutdown complete");
}

impl Default for PluginRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PluginRuntime {
    fn drop(&mut self) {
        self.shutdown();
    }
}
