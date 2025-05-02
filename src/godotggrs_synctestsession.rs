use std::mem;

use crate::*;
use ggrs::{PlayerHandle, PlayerType, SessionBuilder, SyncTestSession};

/// A Godot implementation of [`SyncTestSession`]
#[derive(GodotClass)]
#[class(base=Node)]
pub struct GodotGgrsSyncTestSession {
    base: Base<Node>,
    session_builder: SessionBuilder<GgrsConfig>,
    sess: Option<SyncTestSession<GgrsConfig>>,
    callback_node: Option<Gd<Node>>,
    players: Vec<(PlayerType<<GgrsConfig as Config>::Address>, PlayerHandle)>
}

#[godot_api]
impl INode for GodotGgrsSyncTestSession {
    fn init(base: Base<Node>) -> Self {
        GodotGgrsSyncTestSession {
            base,
            session_builder: SessionBuilder::new(),
            sess: None,
            callback_node: None,
            players: Vec::new()
        }
    }
}

#[godot_api]
impl GodotGgrsSyncTestSession {
    //EXPORTED FUNCTIONS
    #[func]
    fn _ready(&self) {
        godot_print!("GodotGgrsP2PSession _ready() called.");
    }

    /// Adds a local player to the session builder and return the handle.
    /// Cannot add players after calling start_session.
    #[func]
    pub fn add_local_player(&mut self) -> u8 {
        self.add_player(PlayerType::Local) as u8
    }

    /// Set the check distance, measured in frames.
    /// Cannot be set after calling start_session.
    #[func]
    pub fn set_check_distance(&mut self, check_distance: u8) {
        let builder = mem::take(&mut self.session_builder);
        self.session_builder = builder.with_check_distance(check_distance as usize);
    }

    /// Set the maximum prediction window, measured in frames.
    /// Cannot be set after calling start_session.
    /// # Notes
    /// - Max prediction frames is the maximum number of frames GGRS will roll back. Every gamestate older than this is guaranteed to be correct if the players did not desync.
    /// - This value used to default to `8 frames`, but this has been made adjustable with `GGRS 0.7.0`
    #[func]
    pub fn set_max_prediction_window(&mut self, window: u8) {
        let builder = mem::take(&mut self.session_builder);
        self.session_builder = builder.with_max_prediction_window(window as usize);
    }

    /// Sets [SessionBuilder::with_input_delay()]
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn set_input_delay(&mut self, delay: u32) {
        self.session_builder = mem::take(&mut self.session_builder).with_input_delay(delay as usize);
    }

    /// This function will register a player handle's integer-encoded inputs for a subsequent call to advance_frame
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn add_local_input(&mut self, local_player_handle: u8, local_input: u8) {
        match &mut self.sess {
            Some(s) => {
                match s.add_local_input(local_player_handle as PlayerHandle, local_input as <GgrsConfig as Config>::Input) {
                    Err(e) => {
                        godot_error!("{}", e);
                    },
                    _ => ()
                }
            },
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE);
            }
        }
    }

    /// This function will advance the frame
    /// Before using this function you have to set the callback node and make sure it has the following callback functions implemented
    /// - [CALLBACK_FUNC_SAVE_GAME_STATE]
    /// - [CALLBACK_FUNC_LOAD_GAME_STATE]
    /// - [CALLBACK_FUNC_SAVE_GAME_STATE]
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    /// - Will print a [ERR_MESSAGE_NO_CALLBACK_NODE] error if a callback node has not been set
    #[func]
    pub fn advance_frame(&mut self) {
        match &mut self.callback_node {
            Some(callback_node) => match &mut self.sess {
                Some(s) => {
                    match s.advance_frame() {
                        Ok(requests) => {
                            ggrs_request_handlers::handle_requests(callback_node, requests);
                        }
                        Err(e) => {
                            godot_error!("{}", e);
                        }
                    }
                },
                None => {
                    godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE);
                }
            },
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_CALLBACK_NODE);
            }
        }
    }

    /// Calls and returns [SyncTestSession::max_prediction()].
    /// Will return a 0 if no session was made.
    /// # Errors
    /// - Will print an [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn get_max_prediction(&mut self) -> u8 {
        match &mut self.sess {
            Some(s) => s.max_prediction() as u8,
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE);
                return 0;
            }
        }
    }

    /// Sets the callback node that will be called when using [Self::advance_frame()]
    #[func]
    pub fn set_callback_node(&mut self, callback: Gd<Node>) {
        self.callback_node = Some(callback);
    }

    /// Sets the player count and players from [Self::players]. Called internally when starting a session.
    /// # Errors
    /// - Will print a [GgrsError](ggrs::error::GgrsError) error if a player cannot be added.
    fn set_players(&mut self) {
        self.session_builder = mem::take(&mut self.session_builder).with_num_players(self.players.len());
        for (player_type, player_handle) in self.players.iter() {
            match mem::take(&mut self.session_builder).add_player(*player_type, *player_handle) {
                Ok(new_builder) => self.session_builder = new_builder,
                Err(e) => {
                    godot_error!("{}", e);
                }
            }
        }
    }

    /// Sets the callback node that will be called when using [Self::advance_frame()]
    #[func]
    pub fn start_session(&mut self) {
        self.set_players();
        match mem::take(&mut self.session_builder).start_synctest_session() {
            Ok(s) => self.sess = Some(s),
            Err(e) => godot_error!("{}", e)
        }
    }

    #[func]
    pub fn is_running(&self) -> bool {
        self.sess.is_some()
    }

    //NON-EXPORTED FUNCTIONS
    fn add_player(&mut self, player_type: PlayerType<<GgrsConfig as Config>::Address>) -> PlayerHandle {
        let handle = self.players.len();
        self.players.push((player_type, handle));
        handle
    }
}
