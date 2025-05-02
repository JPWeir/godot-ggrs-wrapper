use crate::*;
use arrayref::array_ref;
use ggrs::{Config, Frame, GameStateCell, GgrsRequest, InputStatus};

pub fn handle_requests(callback_node: &mut Gd<Node> , requests: Vec<GgrsRequest<GgrsConfig>>) {
    for item in requests {
        match item {
            GgrsRequest::AdvanceFrame { inputs } => {
                ggrs_request_advance_fame(callback_node, inputs)
            }
            GgrsRequest::LoadGameState { cell, frame } => {
                ggrs_request_load_game_state(callback_node, cell, frame)
            }
            GgrsRequest::SaveGameState { cell, frame } => {
                ggrs_request_save_game_state(callback_node, cell, frame);
            }
        }
    }
}

pub fn ggrs_request_advance_fame(node: &mut Gd<Node>, inputs: Vec<(<GgrsConfig as Config>::Input, InputStatus)>) {
    //Parse parameter inputs in a way that godot can handle then call the callback method
    let mut godot_array: Vec<Variant> = Vec::new();
    for (int_input, _input_status) in inputs {
        godot_array.push(int_input.to_variant());
    }
    node.call(CALLBACK_FUNC_ADVANCE_FRAME, &[godot_array.to_variant()]);
}

fn checksum(packed: &PackedByteArray) -> u128 {
    let blake_hash = blake3::hash(&packed.as_slice());
    // Take the first 16 bytes of the hash, use big endian as our networking standard
    u128::from_be_bytes(*array_ref!(blake_hash.as_bytes(), 0, 16))
}

// TODO: Does the Godot layer really need frame or checksum?
pub fn ggrs_request_load_game_state(node: &mut Gd<Node>, cell: GameStateCell<PackedByteArray>, frame: Frame) {
    //Unpack the cell and have over it's values to godot so it can handle it.
    let state_data = cell.load().unwrap_or_default();
    node.call(CALLBACK_FUNC_LOAD_GAME_STATE, &[state_data.to_variant()]);
}

pub fn ggrs_request_save_game_state(node: &mut Gd<Node>, cell: GameStateCell<PackedByteArray>, frame: Frame) {
    //Store current cell for later use
    let state_variant = node.call(CALLBACK_FUNC_SAVE_GAME_STATE, &[frame.to_variant()]);
    let state_data = PackedByteArray::from_variant(&state_variant);
    
    let checksum: u128 = checksum(&state_data);
    cell.save(frame, Some(state_data), Some(checksum.into()));
}
