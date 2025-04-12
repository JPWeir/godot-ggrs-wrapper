use crate::*;
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

/// computes the fletcher16 checksum, copied from wikipedia: <https://en.wikipedia.org/wiki/Fletcher%27s_checksum>
fn fletcher16(data: &PackedByteArray) -> u16 {
    let mut sum1: u16 = 0;
    let mut sum2: u16 = 0;

    for index in 0..data.len() {
        sum1 = (sum1 + data[index] as u16) % 255;
        sum2 = (sum2 + sum1) % 255;
    }

    (sum2 << 8) | sum1
}

pub fn ggrs_request_load_game_state(node: &mut Gd<Node>, cell: GameStateCell<PackedByteArray>, frame: Frame) {
    //Unpack the cell and have over it's values to godot so it can handle it.
    let state_data = cell.load().unwrap_or_default();
    let checksum = fletcher16(&state_data);
    node.call(CALLBACK_FUNC_LOAD_GAME_STATE, &[frame.to_variant(), state_data.to_variant(), checksum.to_variant()]);
}

pub fn ggrs_request_save_game_state(node: &mut Gd<Node>, cell: GameStateCell<PackedByteArray>, frame: Frame) {
    //Store current cell for later use
    let state_variant: Variant = node.call(CALLBACK_FUNC_SAVE_GAME_STATE, &[frame.to_variant()]);
    let state_data = PackedByteArray::from_variant(&state_variant);
    let checksum = fletcher16(&state_data);
    cell.save(frame, Some(state_data), Some(checksum.into()));
}
