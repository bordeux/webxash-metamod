//! Metamod SDK FFI type definitions.
//!
//! These types are based on the Metamod-P SDK headers:
//! - meta_api.h
//! - plinfo.h
//! - mutil.h

use std::ffi::{c_char, c_float, c_int, c_uchar, c_void};

// =============================================================================
// Forward declarations for opaque engine types
// =============================================================================

/// Opaque edict structure (entity dictionary)
#[repr(C)]
pub struct edict_s {
    _opaque: [u8; 0],
}
pub type edict_t = edict_s;

/// Opaque entvars structure (entity variables)
#[repr(C)]
pub struct entvars_s {
    _opaque: [u8; 0],
}
pub type entvars_t = entvars_s;

/// Opaque cvar structure
#[repr(C)]
pub struct cvar_s {
    pub name: *const c_char,
    pub string: *mut c_char,
    pub flags: c_int,
    pub value: c_float,
    pub next: *mut cvar_s,
}
pub type cvar_t = cvar_s;

// =============================================================================
// Metamod Plugin Info
// =============================================================================

/// When the plugin can be loaded
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginLoadTime {
    Never = 0,
    Startup = 1,
    Changelevel = 2,
    Anytime = 3,
    Anypause = 4,
}

/// Reasons for unloading the plugin
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginUnloadReason {
    Null = 0,
    IniDeleted = 1,
    FileNewer = 2,
    Command = 3,
    CmdForced = 4,
    Delayed = 5,
    PluginRemoved = 6,
    Reload = 7,
}

/// Plugin information structure returned by Meta_Query
#[repr(C)]
pub struct PluginInfo {
    /// Interface version string (e.g., "5:13")
    pub ifvers: *const c_char,
    /// Plugin name
    pub name: *const c_char,
    /// Plugin version
    pub version: *const c_char,
    /// Build date
    pub date: *const c_char,
    /// Author name
    pub author: *const c_char,
    /// URL
    pub url: *const c_char,
    /// Log tag for messages
    pub logtag: *const c_char,
    /// When plugin can be loaded
    pub loadable: PluginLoadTime,
    /// When plugin can be unloaded
    pub unloadable: PluginLoadTime,
}

// SAFETY: PluginInfo contains only static string pointers
unsafe impl Sync for PluginInfo {}
unsafe impl Send for PluginInfo {}

// =============================================================================
// Meta Result Codes
// =============================================================================

/// Result codes for hook functions
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaResult {
    Unset = 0,
    Ignored = 1,
    Handled = 2,
    Override = 3,
    Supercede = 4,
}

// =============================================================================
// Meta Globals
// =============================================================================

/// Global variables passed to plugin by Metamod
#[repr(C)]
pub struct MetaGlobals {
    /// Current hook result
    pub mres: MetaResult,
    /// Previous hook result
    pub prev_mres: MetaResult,
    /// Overall status
    pub status: MetaResult,
    /// Original return value
    pub orig_ret: *mut c_void,
    /// Override return value
    pub override_ret: *mut c_void,
}

// =============================================================================
// Global Variables
// =============================================================================

/// Engine global variables
#[repr(C)]
pub struct GlobalVars {
    pub time: c_float,
    pub frametime: c_float,
    pub force_retouch: c_float,
    pub mapname: c_int, // string_t
    pub startspot: c_int,
    pub deathmatch: c_float,
    pub coop: c_float,
    pub teamplay: c_float,
    pub serverflags: c_float,
    pub found_secrets: c_float,
    pub v_forward: [c_float; 3],
    pub v_up: [c_float; 3],
    pub v_right: [c_float; 3],
    pub trace_allsolid: c_float,
    pub trace_startsolid: c_float,
    pub trace_fraction: c_float,
    pub trace_endpos: [c_float; 3],
    pub trace_plane_normal: [c_float; 3],
    pub trace_plane_dist: c_float,
    pub trace_ent: *mut edict_t,
    pub trace_inopen: c_float,
    pub trace_inwater: c_float,
    pub trace_hitgroup: c_int,
    pub trace_flags: c_int,
    pub msg_entity: c_int,
    pub cd_audio_track: c_int,
    pub max_clients: c_int,
    pub max_entities: c_int,
    pub p_string_base: *const c_char,
    pub p_save_data: *mut c_void,
    pub vec_landmark_offset: [c_float; 3],
}

// =============================================================================
// Engine Functions (partial - add as needed)
// =============================================================================

/// Pointer to function returning int
pub type PfnPrecacheModel = Option<unsafe extern "C" fn(*const c_char) -> c_int>;
pub type PfnPrecacheSound = Option<unsafe extern "C" fn(*const c_char) -> c_int>;
pub type PfnServerPrint = Option<unsafe extern "C" fn(*const c_char)>;
pub type PfnAlertMessage = Option<unsafe extern "C" fn(c_int, *const c_char, ...)>;
pub type PfnCvarRegister = Option<unsafe extern "C" fn(*mut cvar_t)>;
pub type PfnCvarGetPointer = Option<unsafe extern "C" fn(*const c_char) -> *mut cvar_t>;

/// Engine function table (partial)
#[repr(C)]
pub struct EngineFuncs {
    pub pfn_precache_model: PfnPrecacheModel,
    pub pfn_precache_sound: PfnPrecacheSound,
    pub pfn_set_model: Option<unsafe extern "C" fn(*mut edict_t, *const c_char)>,
    pub pfn_model_index: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub pfn_model_frames: Option<unsafe extern "C" fn(c_int) -> c_int>,
    pub pfn_set_size: Option<unsafe extern "C" fn(*mut edict_t, *const c_float, *const c_float)>,
    pub pfn_change_level: Option<unsafe extern "C" fn(*const c_char, *const c_char)>,
    pub pfn_get_spawn_parms: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_save_spawn_parms: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_vec_to_yaw: Option<unsafe extern "C" fn(*const c_float) -> c_float>,
    pub pfn_vec_to_angles: Option<unsafe extern "C" fn(*const c_float, *mut c_float)>,
    pub pfn_move_to_origin: Option<unsafe extern "C" fn(*mut edict_t, *const c_float, c_float, c_int)>,
    pub pfn_change_yaw: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_change_pitch: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_find_entity_by_string: Option<unsafe extern "C" fn(*mut edict_t, *const c_char, *const c_char) -> *mut edict_t>,
    pub pfn_get_entity_illum: Option<unsafe extern "C" fn(*mut edict_t) -> c_int>,
    pub pfn_find_entity_in_sphere: Option<unsafe extern "C" fn(*mut edict_t, *const c_float, c_float) -> *mut edict_t>,
    pub pfn_find_client_in_pvs: Option<unsafe extern "C" fn(*mut edict_t) -> *mut edict_t>,
    pub pfn_entities_in_pvs: Option<unsafe extern "C" fn(*mut edict_t) -> *mut edict_t>,
    pub pfn_make_vectors: Option<unsafe extern "C" fn(*const c_float)>,
    pub pfn_angle_vectors: Option<unsafe extern "C" fn(*const c_float, *mut c_float, *mut c_float, *mut c_float)>,
    pub pfn_create_entity: Option<unsafe extern "C" fn() -> *mut edict_t>,
    pub pfn_remove_entity: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_create_named_entity: Option<unsafe extern "C" fn(c_int) -> *mut edict_t>,
    pub pfn_make_static: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_ent_is_on_floor: Option<unsafe extern "C" fn(*mut edict_t) -> c_int>,
    pub pfn_drop_to_floor: Option<unsafe extern "C" fn(*mut edict_t) -> c_int>,
    pub pfn_walk_move: Option<unsafe extern "C" fn(*mut edict_t, c_float, c_float, c_int) -> c_int>,
    pub pfn_set_origin: Option<unsafe extern "C" fn(*mut edict_t, *const c_float)>,
    pub pfn_emit_sound: Option<unsafe extern "C" fn(*mut edict_t, c_int, *const c_char, c_float, c_float, c_int, c_int)>,
    pub pfn_emit_ambient_sound: Option<unsafe extern "C" fn(*mut edict_t, *mut c_float, *const c_char, c_float, c_float, c_int, c_int)>,
    pub pfn_trace_line: Option<unsafe extern "C" fn(*const c_float, *const c_float, c_int, *mut edict_t, *mut c_void)>,
    pub pfn_trace_toss: Option<unsafe extern "C" fn(*mut edict_t, *mut edict_t, *mut c_void)>,
    pub pfn_trace_monster_hull: Option<unsafe extern "C" fn(*mut edict_t, *const c_float, *const c_float, c_int, *mut edict_t, *mut c_void) -> c_int>,
    pub pfn_trace_hull: Option<unsafe extern "C" fn(*const c_float, *const c_float, c_int, c_int, *mut edict_t, *mut c_void)>,
    pub pfn_trace_model: Option<unsafe extern "C" fn(*const c_float, *const c_float, c_int, *mut edict_t, *mut c_void)>,
    pub pfn_trace_texture: Option<unsafe extern "C" fn(*mut edict_t, *const c_float, *const c_float) -> *const c_char>,
    pub pfn_trace_sphere: Option<unsafe extern "C" fn(*const c_float, *const c_float, c_int, c_float, *mut edict_t, *mut c_void)>,
    pub pfn_get_aim_vector: Option<unsafe extern "C" fn(*mut edict_t, c_float, *mut c_float)>,
    pub pfn_server_command: Option<unsafe extern "C" fn(*const c_char)>,
    pub pfn_server_execute: Option<unsafe extern "C" fn()>,
    pub pfn_client_command: Option<unsafe extern "C" fn(*mut edict_t, *const c_char, ...)>,
    pub pfn_particle_effect: Option<unsafe extern "C" fn(*const c_float, *const c_float, c_float, c_float)>,
    pub pfn_light_style: Option<unsafe extern "C" fn(c_int, *const c_char)>,
    pub pfn_decal_index: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub pfn_point_contents: Option<unsafe extern "C" fn(*const c_float) -> c_int>,
    pub pfn_message_begin: Option<unsafe extern "C" fn(c_int, c_int, *const c_float, *mut edict_t)>,
    pub pfn_message_end: Option<unsafe extern "C" fn()>,
    pub pfn_write_byte: Option<unsafe extern "C" fn(c_int)>,
    pub pfn_write_char: Option<unsafe extern "C" fn(c_int)>,
    pub pfn_write_short: Option<unsafe extern "C" fn(c_int)>,
    pub pfn_write_long: Option<unsafe extern "C" fn(c_int)>,
    pub pfn_write_angle: Option<unsafe extern "C" fn(c_float)>,
    pub pfn_write_coord: Option<unsafe extern "C" fn(c_float)>,
    pub pfn_write_string: Option<unsafe extern "C" fn(*const c_char)>,
    pub pfn_write_entity: Option<unsafe extern "C" fn(c_int)>,
    pub pfn_cvar_register: PfnCvarRegister,
    pub pfn_cvar_get_float: Option<unsafe extern "C" fn(*const c_char) -> c_float>,
    pub pfn_cvar_get_string: Option<unsafe extern "C" fn(*const c_char) -> *const c_char>,
    pub pfn_cvar_set_float: Option<unsafe extern "C" fn(*const c_char, c_float)>,
    pub pfn_cvar_set_string: Option<unsafe extern "C" fn(*const c_char, *const c_char)>,
    pub pfn_alert_message: PfnAlertMessage,
    pub pfn_engine_fprintf: Option<unsafe extern "C" fn(*mut c_void, *const c_char, ...)>,
    pub pfn_alloc_ent_private_data: Option<unsafe extern "C" fn(*mut edict_t, c_int) -> *mut c_void>,
    pub pfn_ent_private_data: Option<unsafe extern "C" fn(*mut edict_t) -> *mut c_void>,
    pub pfn_free_ent_private_data: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_sz_from_index: Option<unsafe extern "C" fn(c_int) -> *const c_char>,
    pub pfn_alloc_string: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub pfn_get_vars_of_ent: Option<unsafe extern "C" fn(*mut edict_t) -> *mut entvars_t>,
    pub pfn_pent_offset_of_pent: Option<unsafe extern "C" fn(*const edict_t) -> c_int>,
    pub pfn_index_of_edict: Option<unsafe extern "C" fn(*const edict_t) -> c_int>,
    pub pfn_pent_of_ent_offset: Option<unsafe extern "C" fn(c_int) -> *mut edict_t>,
    pub pfn_pent_of_ent_index: Option<unsafe extern "C" fn(c_int) -> *mut edict_t>,
    pub pfn_find_entity_by_vars: Option<unsafe extern "C" fn(*mut entvars_t) -> *mut edict_t>,
    pub pfn_get_model_ptr: Option<unsafe extern "C" fn(*mut edict_t) -> *mut c_void>,
    pub pfn_reg_user_msg: Option<unsafe extern "C" fn(*const c_char, c_int) -> c_int>,
    pub pfn_animation_auto_move: Option<unsafe extern "C" fn(*const edict_t, c_float)>,
    pub pfn_get_bone_position: Option<unsafe extern "C" fn(*const edict_t, c_int, *mut c_float, *mut c_float)>,
    pub pfn_function_from_name: Option<unsafe extern "C" fn(*const c_char) -> c_uchar>,
    pub pfn_name_for_function: Option<unsafe extern "C" fn(c_uchar) -> *const c_char>,
    pub pfn_client_printf: Option<unsafe extern "C" fn(*mut edict_t, c_int, *const c_char)>,
    pub pfn_server_print: PfnServerPrint,
    pub pfn_cmd_args: Option<unsafe extern "C" fn() -> *const c_char>,
    pub pfn_cmd_argv: Option<unsafe extern "C" fn(c_int) -> *const c_char>,
    pub pfn_cmd_argc: Option<unsafe extern "C" fn() -> c_int>,
    pub pfn_get_attachment: Option<unsafe extern "C" fn(*const edict_t, c_int, *mut c_float, *mut c_float)>,
    pub pfn_crc32_init: Option<unsafe extern "C" fn(*mut c_uchar)>,
    pub pfn_crc32_process_buffer: Option<unsafe extern "C" fn(*mut c_uchar, *mut c_void, c_int)>,
    pub pfn_crc32_process_byte: Option<unsafe extern "C" fn(*mut c_uchar, c_uchar)>,
    pub pfn_crc32_final: Option<unsafe extern "C" fn(c_uchar) -> c_uchar>,
    pub pfn_random_long: Option<unsafe extern "C" fn(c_int, c_int) -> c_int>,
    pub pfn_random_float: Option<unsafe extern "C" fn(c_float, c_float) -> c_float>,
    pub pfn_set_view: Option<unsafe extern "C" fn(*const edict_t, *const edict_t)>,
    pub pfn_time: Option<unsafe extern "C" fn() -> c_float>,
    pub pfn_crosshair_angle: Option<unsafe extern "C" fn(*const edict_t, c_float, c_float)>,
    pub pfn_load_file_for_me: Option<unsafe extern "C" fn(*const c_char, *mut c_int) -> *mut c_uchar>,
    pub pfn_free_file: Option<unsafe extern "C" fn(*mut c_void)>,
    pub pfn_end_section: Option<unsafe extern "C" fn(*const c_char)>,
    pub pfn_compare_file_time: Option<unsafe extern "C" fn(*const c_char, *const c_char, *mut c_int) -> c_int>,
    pub pfn_get_game_dir: Option<unsafe extern "C" fn(*mut c_char)>,
    pub pfn_cvar_register_variable: Option<unsafe extern "C" fn(*mut cvar_t)>,
    pub pfn_fade_client_volume: Option<unsafe extern "C" fn(*const edict_t, c_int, c_int, c_int, c_int)>,
    pub pfn_set_client_max_speed: Option<unsafe extern "C" fn(*mut edict_t, c_float)>,
    pub pfn_create_fake_client: Option<unsafe extern "C" fn(*const c_char) -> *mut edict_t>,
    pub pfn_run_player_move: Option<unsafe extern "C" fn(*mut edict_t, *const c_float, c_float, c_float, c_float, c_ushort, c_uchar, c_uchar)>,
    pub pfn_number_of_entities: Option<unsafe extern "C" fn() -> c_int>,
    pub pfn_get_info_key_buffer: Option<unsafe extern "C" fn(*mut edict_t) -> *mut c_char>,
    pub pfn_info_key_value: Option<unsafe extern "C" fn(*mut c_char, *const c_char) -> *mut c_char>,
    pub pfn_set_key_value: Option<unsafe extern "C" fn(*mut c_char, *const c_char, *const c_char)>,
    pub pfn_set_client_key_value: Option<unsafe extern "C" fn(c_int, *mut c_char, *const c_char, *const c_char)>,
    pub pfn_is_map_valid: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub pfn_static_decal: Option<unsafe extern "C" fn(*const c_float, c_int, c_int, c_int)>,
    pub pfn_precache_generic: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub pfn_get_player_userid: Option<unsafe extern "C" fn(*mut edict_t) -> c_int>,
    pub pfn_build_sound_msg: Option<unsafe extern "C" fn(*mut edict_t, c_int, *const c_char, c_float, c_float, c_int, c_int, c_int, c_int, *const c_float, *mut edict_t)>,
    pub pfn_is_dedicated_server: Option<unsafe extern "C" fn() -> c_int>,
    pub pfn_cvar_get_pointer: PfnCvarGetPointer,
    pub pfn_get_player_wonid: Option<unsafe extern "C" fn(*mut edict_t) -> c_uint>,
    pub pfn_info_remove_key: Option<unsafe extern "C" fn(*mut c_char, *const c_char)>,
    pub pfn_get_physics_key_value: Option<unsafe extern "C" fn(*const edict_t, *const c_char) -> *const c_char>,
    pub pfn_set_physics_key_value: Option<unsafe extern "C" fn(*const edict_t, *const c_char, *const c_char)>,
    pub pfn_get_physics_info_string: Option<unsafe extern "C" fn(*const edict_t) -> *const c_char>,
    pub pfn_precache_event: Option<unsafe extern "C" fn(c_int, *const c_char) -> c_ushort>,
    pub pfn_playback_event: Option<unsafe extern "C" fn(c_int, *const edict_t, c_ushort, c_float, *const c_float, *const c_float, c_float, c_float, c_int, c_int, c_int, c_int)>,
    pub pfn_set_fat_pvs: Option<unsafe extern "C" fn(*mut c_float) -> *mut c_uchar>,
    pub pfn_set_fat_pas: Option<unsafe extern "C" fn(*mut c_float) -> *mut c_uchar>,
    pub pfn_check_visibility: Option<unsafe extern "C" fn(*const edict_t, *mut c_uchar) -> c_int>,
    pub pfn_delta_set_field: Option<unsafe extern "C" fn(*mut c_void, *const c_char)>,
    pub pfn_delta_unset_field: Option<unsafe extern "C" fn(*mut c_void, *const c_char)>,
    pub pfn_delta_add_encoder: Option<unsafe extern "C" fn(*const c_char, Option<unsafe extern "C" fn(*mut c_void, *const c_uchar, *const c_uchar)>)>,
    pub pfn_get_current_player: Option<unsafe extern "C" fn() -> c_int>,
    pub pfn_can_skip_player: Option<unsafe extern "C" fn(*const edict_t) -> c_int>,
    pub pfn_delta_find_field: Option<unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int>,
    pub pfn_delta_set_field_by_index: Option<unsafe extern "C" fn(*mut c_void, c_int)>,
    pub pfn_delta_unset_field_by_index: Option<unsafe extern "C" fn(*mut c_void, c_int)>,
    pub pfn_set_group_mask: Option<unsafe extern "C" fn(c_int, c_int)>,
    pub pfn_engine_stub1: Option<unsafe extern "C" fn(c_int, *const c_char) -> c_int>,
    pub pfn_engine_stub2: Option<unsafe extern "C" fn()>,
    pub pfn_voice_get_client_listening: Option<unsafe extern "C" fn(c_int, c_int) -> c_int>,
    pub pfn_voice_set_client_listening: Option<unsafe extern "C" fn(c_int, c_int, c_int) -> c_int>,
    pub pfn_get_player_auth_id: Option<unsafe extern "C" fn(*mut edict_t) -> *const c_char>,
    pub pfn_sequence_get: Option<unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_void>,
    pub pfn_sequence_pickup_sentence: Option<unsafe extern "C" fn(*const c_char, c_int, *mut c_int) -> *mut c_void>,
    pub pfn_get_file_size: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub pfn_get_approx_wave_play_len: Option<unsafe extern "C" fn(*const c_char) -> c_uint>,
    pub pfn_is_career_match: Option<unsafe extern "C" fn() -> c_int>,
    pub pfn_get_localized_string_length: Option<unsafe extern "C" fn(*const c_char) -> c_int>,
    pub pfn_register_tutor_message_shown: Option<unsafe extern "C" fn(c_int)>,
    pub pfn_get_times_tutor_message_shown: Option<unsafe extern "C" fn(c_int) -> c_int>,
    pub pfn_process_tutor_message_decay_buffer: Option<unsafe extern "C" fn(*mut c_int, c_int)>,
    pub pfn_construct_tutor_message_decay_buffer: Option<unsafe extern "C" fn(*mut c_int, c_int)>,
    pub pfn_reset_tutor_message_decay_data: Option<unsafe extern "C" fn()>,
    pub pfn_query_client_cvar_value: Option<unsafe extern "C" fn(*const edict_t, *const c_char)>,
    pub pfn_query_client_cvar_value2: Option<unsafe extern "C" fn(*const edict_t, *const c_char, c_int)>,
    pub pfn_check_parm: Option<unsafe extern "C" fn(*const c_char, *mut *mut c_char) -> c_int>,
}

use std::os::raw::c_uint;
use std::os::raw::c_ushort;

// =============================================================================
// DLL Functions
// =============================================================================

/// Game DLL function table
#[repr(C)]
pub struct DllFunctions {
    pub pfn_game_init: Option<unsafe extern "C" fn()>,
    pub pfn_spawn: Option<unsafe extern "C" fn(*mut edict_t) -> c_int>,
    pub pfn_think: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_use: Option<unsafe extern "C" fn(*mut edict_t, *mut edict_t)>,
    pub pfn_touch: Option<unsafe extern "C" fn(*mut edict_t, *mut edict_t)>,
    pub pfn_blocked: Option<unsafe extern "C" fn(*mut edict_t, *mut edict_t)>,
    pub pfn_keyvalue: Option<unsafe extern "C" fn(*mut edict_t, *mut c_void)>,
    pub pfn_save: Option<unsafe extern "C" fn(*mut edict_t, *mut c_void)>,
    pub pfn_restore: Option<unsafe extern "C" fn(*mut edict_t, *mut c_void, c_int) -> c_int>,
    pub pfn_set_abs_box: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_save_write_fields: Option<unsafe extern "C" fn(*mut c_void, *const c_char, *mut c_void, *mut c_void, c_int)>,
    pub pfn_save_read_fields: Option<unsafe extern "C" fn(*mut c_void, *const c_char, *mut c_void, *mut c_void, c_int)>,
    pub pfn_save_global_state: Option<unsafe extern "C" fn(*mut c_void)>,
    pub pfn_restore_global_state: Option<unsafe extern "C" fn(*mut c_void)>,
    pub pfn_reset_global_state: Option<unsafe extern "C" fn()>,
    pub pfn_client_connect: Option<unsafe extern "C" fn(*mut edict_t, *const c_char, *const c_char, *mut c_char) -> c_int>,
    pub pfn_client_disconnect: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_client_kill: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_client_put_in_server: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_client_command: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_client_user_info_changed: Option<unsafe extern "C" fn(*mut edict_t, *mut c_char)>,
    pub pfn_server_activate: Option<unsafe extern "C" fn(*mut edict_t, c_int, c_int)>,
    pub pfn_server_deactivate: Option<unsafe extern "C" fn()>,
    pub pfn_player_pre_think: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_player_post_think: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_start_frame: Option<unsafe extern "C" fn()>,
    pub pfn_params_new_level: Option<unsafe extern "C" fn()>,
    pub pfn_params_change_level: Option<unsafe extern "C" fn()>,
    pub pfn_get_game_description: Option<unsafe extern "C" fn() -> *const c_char>,
    pub pfn_player_customization: Option<unsafe extern "C" fn(*mut edict_t, *mut c_void)>,
    pub pfn_spectator_connect: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_spectator_disconnect: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_spectator_think: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_sys_error: Option<unsafe extern "C" fn(*const c_char)>,
    pub pfn_pm_move: Option<unsafe extern "C" fn(*mut c_void, c_int)>,
    pub pfn_pm_init: Option<unsafe extern "C" fn(*mut c_void)>,
    pub pfn_pm_find_texture_type: Option<unsafe extern "C" fn(*mut c_char) -> c_char>,
    pub pfn_setup_visibility: Option<unsafe extern "C" fn(*mut edict_t, *mut edict_t, *mut *mut c_uchar, *mut *mut c_uchar)>,
    pub pfn_update_client_data: Option<unsafe extern "C" fn(*const edict_t, c_int, *mut c_void)>,
    pub pfn_add_to_full_pack: Option<unsafe extern "C" fn(*mut c_void, c_int, *mut edict_t, *mut edict_t, c_int, c_int, *mut c_uchar) -> c_int>,
    pub pfn_create_baseline: Option<unsafe extern "C" fn(c_int, c_int, *mut c_void, *mut edict_t, c_int, *const c_float, *const c_float)>,
    pub pfn_register_encoders: Option<unsafe extern "C" fn()>,
    pub pfn_get_weapon_data: Option<unsafe extern "C" fn(*mut edict_t, *mut c_void) -> c_int>,
    pub pfn_cmd_start: Option<unsafe extern "C" fn(*const edict_t, *const c_void, c_uint)>,
    pub pfn_cmd_end: Option<unsafe extern "C" fn(*const edict_t)>,
    pub pfn_connection_less_packet: Option<unsafe extern "C" fn(*const c_void, *const c_char, *mut c_char, *mut c_int) -> c_int>,
    pub pfn_get_hull_bounds: Option<unsafe extern "C" fn(c_int, *mut c_float, *mut c_float) -> c_int>,
    pub pfn_create_instanced_baselines: Option<unsafe extern "C" fn()>,
    pub pfn_inconsistent_file: Option<unsafe extern "C" fn(*const edict_t, *const c_char, *mut c_char) -> c_int>,
    pub pfn_allow_lag_compensation: Option<unsafe extern "C" fn() -> c_int>,
}

/// New DLL function table
#[repr(C)]
pub struct NewDllFunctions {
    pub pfn_on_free_ent_private_data: Option<unsafe extern "C" fn(*mut edict_t)>,
    pub pfn_game_shutdown: Option<unsafe extern "C" fn()>,
    pub pfn_should_collide: Option<unsafe extern "C" fn(*mut edict_t, *mut edict_t) -> c_int>,
    pub pfn_cvar_value: Option<unsafe extern "C" fn(*const edict_t, *const c_char)>,
    pub pfn_cvar_value2: Option<unsafe extern "C" fn(*const edict_t, c_int, *const c_char, *const c_char)>,
}

// =============================================================================
// Meta Functions Table
// =============================================================================

/// Function pointer types for Meta functions table
pub type GetEntityApiFn = unsafe extern "C" fn(*mut DllFunctions, c_int) -> c_int;
pub type GetEntityApi2Fn = unsafe extern "C" fn(*mut DllFunctions, *mut c_int) -> c_int;
pub type GetNewDllFunctionsFn = unsafe extern "C" fn(*mut NewDllFunctions, *mut c_int) -> c_int;
pub type GetEngineFunctionsFn = unsafe extern "C" fn(*mut EngineFuncs, *mut c_int) -> c_int;

/// Meta functions table filled by plugin
#[repr(C)]
pub struct MetaFunctions {
    pub pfn_get_entity_api: Option<GetEntityApiFn>,
    pub pfn_get_entity_api_post: Option<GetEntityApiFn>,
    pub pfn_get_entity_api2: Option<GetEntityApi2Fn>,
    pub pfn_get_entity_api2_post: Option<GetEntityApi2Fn>,
    pub pfn_get_new_dll_functions: Option<GetNewDllFunctionsFn>,
    pub pfn_get_new_dll_functions_post: Option<GetNewDllFunctionsFn>,
    pub pfn_get_engine_functions: Option<GetEngineFunctionsFn>,
    pub pfn_get_engine_functions_post: Option<GetEngineFunctionsFn>,
}

// =============================================================================
// Meta Utility Functions
// =============================================================================

/// Log levels for MetaUtilFuncs::log_message
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginLogLevel {
    Info = 0,
    Error = 1,
    Debug = 2,
}

/// Utility functions provided by Metamod
#[repr(C)]
pub struct MetaUtilFuncs {
    pub pfn_log_console: Option<unsafe extern "C" fn(*const PluginInfo, *const c_char, ...)>,
    pub pfn_log_message: Option<unsafe extern "C" fn(*const PluginInfo, *const c_char, ...)>,
    pub pfn_log_error: Option<unsafe extern "C" fn(*const PluginInfo, *const c_char, ...)>,
    pub pfn_log_developer: Option<unsafe extern "C" fn(*const PluginInfo, *const c_char, ...)>,
    pub pfn_center_say: Option<unsafe extern "C" fn(*const PluginInfo, *const c_char, ...)>,
    pub pfn_center_say_parms: Option<unsafe extern "C" fn(*const PluginInfo, *mut c_void, *const c_char, ...)>,
    pub pfn_center_say_varargs: Option<unsafe extern "C" fn(*const PluginInfo, *mut c_void, *const c_char, *mut c_void)>,
    pub pfn_call_game_entity: Option<unsafe extern "C" fn(*const PluginInfo, *const c_char, *mut entvars_t) -> c_int>,
    pub pfn_get_user_msg_id: Option<unsafe extern "C" fn(*const PluginInfo, *const c_char, *mut c_int) -> c_int>,
    pub pfn_get_user_msg_name: Option<unsafe extern "C" fn(*const PluginInfo, c_int, *mut c_int) -> *const c_char>,
    pub pfn_get_plugin_path: Option<unsafe extern "C" fn(*const PluginInfo) -> *const c_char>,
    pub pfn_get_game_info: Option<unsafe extern "C" fn(*const PluginInfo, c_int) -> *const c_char>,
    pub pfn_load_plugin: Option<unsafe extern "C" fn(*const PluginInfo, *const c_char, PluginLoadTime, *mut *mut c_void) -> c_int>,
    pub pfn_unload_plugin: Option<unsafe extern "C" fn(*const PluginInfo, *const c_char, PluginLoadTime, PluginUnloadReason) -> c_int>,
    pub pfn_unload_plugin_by_handle: Option<unsafe extern "C" fn(*const PluginInfo, *mut c_void, PluginLoadTime, PluginUnloadReason) -> c_int>,
    pub pfn_is_querying_client_cvar: Option<unsafe extern "C" fn(*const PluginInfo, *const edict_t) -> *const c_char>,
    pub pfn_make_request_id: Option<unsafe extern "C" fn(*const PluginInfo) -> c_int>,
    pub pfn_get_hook_tables: Option<unsafe extern "C" fn(*const PluginInfo, *mut *mut EngineFuncs, *mut *mut DllFunctions, *mut *mut NewDllFunctions)>,
}

// =============================================================================
// GameDLL Funcs wrapper
// =============================================================================

/// Pointers to game DLL function tables
#[repr(C)]
pub struct GameDllFuncs {
    pub dllapi_table: *mut DllFunctions,
    pub newapi_table: *mut NewDllFunctions,
}
