//! Metamod plugin exported functions.
//!
//! These are the C functions that Metamod expects to find in the plugin DLL.

use std::ffi::{c_char, c_int};
use std::ptr;

use super::types::*;
use crate::plugin::PLUGIN;

// =============================================================================
// Plugin Information
// =============================================================================

/// Metamod interface version
const META_INTERFACE_VERSION: &[u8] = b"5:13\0";

/// Plugin version (null-terminated)
const PLUGIN_VERSION: &[u8] = concat!(env!("CARGO_PKG_VERSION"), "\0").as_bytes();

/// Static plugin information - must remain valid for the plugin's lifetime
#[no_mangle]
pub static PLUGIN_INFO: PluginInfo = PluginInfo {
    ifvers: META_INTERFACE_VERSION.as_ptr().cast::<c_char>(),
    name: b"WebXash3D WebRTC Bridge\0".as_ptr().cast::<c_char>(),
    version: PLUGIN_VERSION.as_ptr().cast::<c_char>(),
    date: b"2025\0".as_ptr().cast::<c_char>(),
    author: b"webxash3d\0".as_ptr().cast::<c_char>(),
    url: b"https://github.com/bordeux/webxash3d-metamod\0".as_ptr().cast::<c_char>(),
    logtag: b"WEBXASH\0".as_ptr().cast::<c_char>(),
    loadable: PluginLoadTime::Startup,
    unloadable: PluginLoadTime::Anypause,
};

// =============================================================================
// Metamod API Functions
// =============================================================================

/// Called before GiveFnptrsToDll to identify this as a Metamod plugin.
#[no_mangle]
pub extern "C" fn Meta_Init() {
    // Early initialization - called before we have engine functions
    PLUGIN.lock().pre_init();
}

/// Query plugin information.
///
/// # Safety
/// Called by Metamod with valid pointers.
#[no_mangle]
pub unsafe extern "C" fn Meta_Query(
    _interface_version: *const c_char,
    plugin_info: *mut *const PluginInfo,
    meta_util_funcs: *mut MetaUtilFuncs,
) -> c_int {
    if plugin_info.is_null() {
        return 0;
    }

    // Return our plugin info
    *plugin_info = &PLUGIN_INFO;

    // Store utility functions for logging
    if !meta_util_funcs.is_null() {
        PLUGIN.lock().set_meta_util_funcs(meta_util_funcs);
    }

    1 // TRUE - success
}

/// Attach plugin to Metamod.
///
/// # Safety
/// Called by Metamod with valid pointers.
#[no_mangle]
pub unsafe extern "C" fn Meta_Attach(
    _now: PluginLoadTime,
    function_table: *mut MetaFunctions,
    meta_globals: *mut MetaGlobals,
    gamedll_funcs: *mut GameDllFuncs,
) -> c_int {
    if function_table.is_null() {
        return 0;
    }

    // Fill in our function table
    (*function_table).pfn_get_entity_api = None;
    (*function_table).pfn_get_entity_api_post = None;
    (*function_table).pfn_get_entity_api2 = Some(get_entity_api2);
    (*function_table).pfn_get_entity_api2_post = None;
    (*function_table).pfn_get_new_dll_functions = None;
    (*function_table).pfn_get_new_dll_functions_post = None;
    (*function_table).pfn_get_engine_functions = None;
    (*function_table).pfn_get_engine_functions_post = None;

    // Store globals (server will start on first map load in on_server_activate)
    let mut plugin = PLUGIN.lock();
    plugin.set_meta_globals(meta_globals);
    plugin.set_gamedll_funcs(gamedll_funcs);

    1 // TRUE - success
}

/// Detach plugin from Metamod.
#[no_mangle]
pub extern "C" fn Meta_Detach(
    _now: PluginLoadTime,
    _reason: PluginUnloadReason,
) -> c_int {
    // Shutdown the WebRTC server
    PLUGIN.lock().shutdown();

    1 // TRUE - success
}

/// Provide engine function pointers to the plugin.
///
/// # Safety
/// Called by Metamod with valid pointers.
#[no_mangle]
pub unsafe extern "C" fn GiveFnptrsToDll(
    engine_funcs: *mut EngineFuncs,
    global_vars: *mut GlobalVars,
) {
    PLUGIN.lock().set_engine_funcs(engine_funcs, global_vars);
}

// =============================================================================
// DLL API Functions
// =============================================================================

/// Our DLL functions table - we only hook what we need
static mut G_DLL_FUNCS: DllFunctions = DllFunctions {
    pfn_game_init: Some(game_init),
    pfn_spawn: None,
    pfn_think: None,
    pfn_use: None,
    pfn_touch: None,
    pfn_blocked: None,
    pfn_keyvalue: None,
    pfn_save: None,
    pfn_restore: None,
    pfn_set_abs_box: None,
    pfn_save_write_fields: None,
    pfn_save_read_fields: None,
    pfn_save_global_state: None,
    pfn_restore_global_state: None,
    pfn_reset_global_state: None,
    pfn_client_connect: None,
    pfn_client_disconnect: None,
    pfn_client_kill: None,
    pfn_client_put_in_server: None,
    pfn_client_command: None,
    pfn_client_user_info_changed: None,
    pfn_server_activate: Some(server_activate),
    pfn_server_deactivate: Some(server_deactivate),
    pfn_player_pre_think: None,
    pfn_player_post_think: None,
    pfn_start_frame: None,
    pfn_params_new_level: None,
    pfn_params_change_level: None,
    pfn_get_game_description: None,
    pfn_player_customization: None,
    pfn_spectator_connect: None,
    pfn_spectator_disconnect: None,
    pfn_spectator_think: None,
    pfn_sys_error: None,
    pfn_pm_move: None,
    pfn_pm_init: None,
    pfn_pm_find_texture_type: None,
    pfn_setup_visibility: None,
    pfn_update_client_data: None,
    pfn_add_to_full_pack: None,
    pfn_create_baseline: None,
    pfn_register_encoders: None,
    pfn_get_weapon_data: None,
    pfn_cmd_start: None,
    pfn_cmd_end: None,
    pfn_connection_less_packet: None,
    pfn_get_hull_bounds: None,
    pfn_create_instanced_baselines: None,
    pfn_inconsistent_file: None,
    pfn_allow_lag_compensation: None,
};

/// Get entity API version 2.
///
/// # Safety
/// Called by Metamod with valid pointers.
#[no_mangle]
pub unsafe extern "C" fn get_entity_api2(
    func_table: *mut DllFunctions,
    interface_version: *mut c_int,
) -> c_int {
    if func_table.is_null() || interface_version.is_null() {
        return 0;
    }

    // Copy our function table
    ptr::copy_nonoverlapping(&raw const G_DLL_FUNCS, func_table, 1);

    1 // TRUE - success
}

// =============================================================================
// Hook Implementations
// =============================================================================

/// Called when the game initializes.
unsafe extern "C" fn game_init() {
    PLUGIN.lock().on_game_init();
}

/// Called when the server activates (map load).
unsafe extern "C" fn server_activate(
    _edict_list: *mut edict_t,
    _edict_count: c_int,
    _client_max: c_int,
) {
    PLUGIN.lock().on_server_activate();
}

/// Called when the server deactivates (map unload).
unsafe extern "C" fn server_deactivate() {
    PLUGIN.lock().on_server_deactivate();
}
