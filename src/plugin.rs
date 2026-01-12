//! Plugin state management.
//!
//! Manages the lifecycle of the WebRTC server and integration with Metamod.

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use std::ptr;

use crate::config::PluginConfig;
use crate::metamod::types::*;
use crate::runtime::PluginRuntime;

/// HLDS hostport cvar
const CVAR_HOSTPORT: &[u8] = b"hostport\0";

/// Global plugin instance
pub static PLUGIN: Lazy<Mutex<Plugin>> = Lazy::new(|| Mutex::new(Plugin::new()));

/// Plugin state
pub struct Plugin {
    /// Engine function pointers
    engine_funcs: *mut EngineFuncs,
    /// Global variables
    global_vars: *mut GlobalVars,
    /// Meta globals
    meta_globals: *mut MetaGlobals,
    /// Meta utility functions
    meta_util_funcs: *mut MetaUtilFuncs,
    /// Game DLL functions
    gamedll_funcs: *mut GameDllFuncs,
    /// Async runtime for WebRTC server
    runtime: Option<PluginRuntime>,
    /// Plugin configuration
    config: PluginConfig,
    /// Whether the server is running
    running: bool,
}

// SAFETY: Plugin is only accessed through a Mutex
unsafe impl Send for Plugin {}
unsafe impl Sync for Plugin {}

impl Plugin {
    /// Create a new plugin instance.
    pub fn new() -> Self {
        Self {
            engine_funcs: ptr::null_mut(),
            global_vars: ptr::null_mut(),
            meta_globals: ptr::null_mut(),
            meta_util_funcs: ptr::null_mut(),
            gamedll_funcs: ptr::null_mut(),
            runtime: None,
            config: PluginConfig::default(),
            running: false,
        }
    }

    /// Pre-initialization called from Meta_Init.
    pub fn pre_init(&mut self) {
        // Called before we have engine functions
        // Initialize tracing or other early setup here
    }

    /// Set engine function pointers.
    ///
    /// # Safety
    /// Pointers must be valid for the lifetime of the plugin.
    pub unsafe fn set_engine_funcs(
        &mut self,
        engine_funcs: *mut EngineFuncs,
        global_vars: *mut GlobalVars,
    ) {
        self.engine_funcs = engine_funcs;
        self.global_vars = global_vars;
    }

    /// Set meta utility functions.
    ///
    /// # Safety
    /// Pointer must be valid for the lifetime of the plugin.
    pub unsafe fn set_meta_util_funcs(&mut self, funcs: *mut MetaUtilFuncs) {
        self.meta_util_funcs = funcs;
    }

    /// Set meta globals.
    ///
    /// # Safety
    /// Pointer must be valid for the lifetime of the plugin.
    pub unsafe fn set_meta_globals(&mut self, globals: *mut MetaGlobals) {
        self.meta_globals = globals;
    }

    /// Set game DLL functions.
    ///
    /// # Safety
    /// Pointer must be valid for the lifetime of the plugin.
    pub unsafe fn set_gamedll_funcs(&mut self, funcs: *mut GameDllFuncs) {
        self.gamedll_funcs = funcs;
    }

    /// Start the WebRTC server.
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.running {
            return Ok(());
        }

        // Load configuration from cvars
        self.config = self.load_config();

        self.log_info(&format!(
            "Starting WebRTC server on port {}",
            self.config.http_port
        ));

        // Create and start the runtime
        let mut runtime = PluginRuntime::new();
        runtime.start(self.config.clone())?;
        self.runtime = Some(runtime);
        self.running = true;

        self.log_info("WebRTC server started successfully");

        Ok(())
    }

    /// Shutdown the WebRTC server.
    pub fn shutdown(&mut self) {
        if !self.running {
            return;
        }

        self.log_info("Shutting down WebRTC server");

        if let Some(mut runtime) = self.runtime.take() {
            runtime.shutdown();
        }

        self.running = false;
        self.log_info("WebRTC server stopped");
    }

    /// Called when the game initializes.
    pub fn on_game_init(&mut self) {
        self.register_cvars();
    }

    /// Called when the server activates (map load).
    pub fn on_server_activate(&mut self) {
        // Start server on first map load (hostport cvar is now available)
        if !self.running {
            if let Err(e) = self.start() {
                self.log_error(&format!("Failed to start server: {e}"));
            }
        } else {
            // Reload config on map change
            self.config = self.load_config();
        }
    }

    /// Called when the server deactivates (map unload).
    pub fn on_server_deactivate(&mut self) {
        // Nothing to do here for now
    }

    /// Register plugin cvars.
    fn register_cvars(&mut self) {
        // CVars are optional - hostport is built-in to HLDS
    }

    /// Load configuration from cvars.
    fn load_config(&self) -> PluginConfig {
        let mut config = PluginConfig::default();

        if self.engine_funcs.is_null() {
            return config;
        }

        // SAFETY: engine_funcs checked above
        unsafe {
            let funcs = &*self.engine_funcs;
            let cvar_name = CVAR_HOSTPORT.as_ptr().cast();

            // Use pfn_cvar_get_pointer to read hostport cvar
            if let Some(cvar_get_pointer) = funcs.pfn_cvar_get_pointer {
                let cvar_ptr = cvar_get_pointer(cvar_name);

                if !cvar_ptr.is_null() {
                    let cvar = &*cvar_ptr;

                    if cvar.value > 0.0 && cvar.value < 65536.0 {
                        config.http_port = cvar.value as u16;
                        config.game_port = cvar.value as u16;
                    }
                }
            }
        }

        config
    }

    /// Log an info message.
    pub fn log_info(&self, msg: &str) {
        self.server_print(&format!("[WEBXASH] {msg}\n"));
    }

    /// Log an error message.
    pub fn log_error(&self, msg: &str) {
        self.server_print(&format!("[WEBXASH] ERROR: {msg}\n"));
    }

    /// Print to server console.
    fn server_print(&self, msg: &str) {
        if self.engine_funcs.is_null() {
            // Fallback to stderr if no engine
            eprintln!("{msg}");
            return;
        }

        // SAFETY: We check for null above
        unsafe {
            if let Some(print_fn) = (*self.engine_funcs).pfn_server_print {
                // Create null-terminated string
                let c_msg = std::ffi::CString::new(msg).unwrap_or_default();
                print_fn(c_msg.as_ptr());
            }
        }
    }

    /// Get the game server address (localhost).
    pub fn game_server_addr(&self) -> String {
        // The game server is running on the same machine
        // Default HLDS port is 27015
        format!("127.0.0.1:{}", self.config.game_port)
    }
}

impl Default for Plugin {
    fn default() -> Self {
        Self::new()
    }
}
