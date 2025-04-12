#![warn(missing_docs)]
//! # Godot-GGRS-Wrapper
//! Godot-GGRS-Wrapper exposes different functions to interact with GGRS inside Godot.
//! All documentation written is explicitly targeted towards use inside Godot, any functions that are usable in Godot are labeld with #[func].
//! For example the [GodotGgrsP2PSession::add_remote_player()] method would just be used like this in Godot: `p2p.add_remote_player("127.0.0.1:7070")`.

use std::net::SocketAddr;
use ggrs::Config;
use godot::prelude::*;

mod ggrs_request_handlers;
mod godotggrs_p2psession;
mod godotggrs_p2pspectatorsession;
mod godotggrs_synctestsession;

/// Error message that is printed when there's no GGRS session made.
pub const ERR_MESSAGE_NO_SESSION_MADE: &str = "No session was made.";
/// Error message that is printed when there's no callback node specified.
pub const ERR_MESSAGE_NO_CALLBACK_NODE: &str = "No callback node was specified.";
/// The name of the Godot callback function that gets called when requesting a state save.
pub const CALLBACK_FUNC_SAVE_GAME_STATE: &str = "ggrs_save_game_state";
/// The name of the Godot callback function that gets called when requesting a state load.
pub const CALLBACK_FUNC_LOAD_GAME_STATE: &str = "ggrs_load_game_state";
/// The name of the Godot callback function that gets called when requesting to advance the frame.
pub const CALLBACK_FUNC_ADVANCE_FRAME: &str = "ggrs_advance_frame";

/// Implementation of [Config] for use in Godot.
#[derive(Debug)]
pub struct GgrsConfig;
impl Config for GgrsConfig {
    type Input = u8;
    type State = PackedByteArray;
    type Address = SocketAddr;
}

struct GodotGGRS;

#[gdextension]
unsafe impl ExtensionLibrary for GodotGGRS {}