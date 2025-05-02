use crate::*;
use ggrs::{Config, Frame, GgrsEvent, P2PSession, PlayerHandle, PlayerType, SessionBuilder, SessionState, UdpNonBlockingSocket};
use std::mem;

/// A Godot implementation of [`P2PSession`]
#[derive(GodotClass)]
#[class(base=Node)]
pub struct GodotGgrsP2PSession {
    base: Base<Node>,
    session_builder: SessionBuilder<GgrsConfig>,
    sess: Option<P2PSession<GgrsConfig>>,
    callback_node: Option<Gd<Node>>,
    players: Vec<(PlayerType<<GgrsConfig as Config>::Address>, PlayerHandle)>
}

#[godot_api]
impl INode for GodotGgrsP2PSession {
    fn init(base: Base<Node>) -> Self {
        GodotGgrsP2PSession {
            base,
            session_builder: SessionBuilder::new(),
            sess: None,
            callback_node: None,
            players: Vec::new()
        }
    }
}

#[godot_api]
impl GodotGgrsP2PSession {
    //EXPORTED FUNCTIONS
    #[func]
    fn _ready(&self) {
        godot_print!("GodotGgrsP2PSession _ready() called.");
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

    /// Adds a local player to the session builder and return the handle.
    /// Cannot add players after calling start_session.
    #[func]
    pub fn add_local_player(&mut self) -> u8 {
        self.add_player(PlayerType::Local) as u8
    }

    /// Adds a remote player to the session builder and returns the handle.
    /// Cannot add players after calling start_session.
    /// # Example
    /// The following example shows how to format an address string, starting with the IP and ending with the port.
    /// ```
    /// p2p.add_remote_player("127.0.0.1:7070")
    /// ```
    /// # Errors
    /// - Will panic if the address string could not be converted to an [std::net::SocketAddr]
    #[func]
    pub fn add_remote_player(&mut self, address: String) -> u8 {
        let remote_addr: std::net::SocketAddr = address.parse().unwrap();
        self.add_player(PlayerType::Remote(remote_addr)) as u8
    }

    /// Adds a spectator to the session and returns the handle
    /// Cannot add players after calling start_session.
    /// # Errors
    /// - Will panic if the address string could not be converted to an [std::net::SocketAddr]
    #[func]
    pub fn add_spectator(&mut self, address: String) -> u8 {
        let remote_addr: std::net::SocketAddr = address.parse().unwrap();
        self.add_player(PlayerType::Spectator(remote_addr)) as u8
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

    /// Starts a [P2PSession]
    /// # Errors
    /// - Will print a [GgrsError] error if no session is made.
    /// - Will print a [std::io::Error] if the local_port cannot be bound.
    #[func]
    pub fn start_session(&mut self, local_port: u16) {
        self.set_players();
        match UdpNonBlockingSocket::bind_to_port(local_port) {
            Ok(socket) => {
                match mem::take(&mut self.session_builder).start_p2p_session(socket) {
                    Ok(session) => {
                        self.sess = Some(session);
                        godot_print!("Started GodotGGRS session")
                    },
                    Err(e) => {
                        godot_error!("{}", e);
                    }
                }
            },
            Err(e) => {
                godot_error!("{}", e);
            }
        };
    }

    /// Returns true if connection has been established with remote players and is ready to start taking inputs via [Self::advance_frame()]
    #[func]
    pub fn is_running(&self) -> bool {
        match &self.sess {
            Some(s) => s.current_state() == SessionState::Running,
            None => false,
        }
    }

    /// Returns the current sate of the session as a String. Take a look at [SessionState] for all possible states.
    #[func]
    pub fn get_current_state(&mut self) -> String {
        match &mut self.sess {
            Some(s) => match s.current_state() {
                SessionState::Running => "Running".to_owned(),
                SessionState::Synchronizing => "Synchronizing".to_owned(),
            },
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE);
                "".to_owned()
            }
        }
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

    /// Sets the session's FPS.
    /// # Errors
    /// - Will print a [GgrsError](ggrs::error::GgrsError) error if the FPS is zero.
    #[func]
    pub fn set_fps(&mut self, fps: u8) {
        let builder = mem::take(&mut self.session_builder);
        match builder.with_fps(fps as usize) {
            Ok(new_builder) => self.session_builder = new_builder,
            Err(e) => godot_error!("{}", e),
        }
    }

    /// Sets the callback node that will be called when using [Self::advance_frame()]
    #[func]
    pub fn set_callback_node(&mut self, callback: Gd<Node>) {
        self.callback_node = Some(callback);
    }

    /// Calls [P2PSession::poll_remote_clients()]
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn poll_remote_clients(&mut self) {
        match &mut self.sess {
            Some(s) => s.poll_remote_clients(),
            None => godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE),
        }
    }

    /// Prints out network stats of specified handle
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn print_network_stats(&mut self, handle: u8) {
        match &mut self.sess {
            Some(s) => match s.network_stats(handle as PlayerHandle) {
                Ok(n) => godot_print!("send_queue_len: {0}; ping: {1}; kbps_sent: {2}; local_frames_behind: {3}; remote_frames_behind: {4};", n.send_queue_len, n.ping, n.kbps_sent, n.local_frames_behind, n.remote_frames_behind),
                Err(e) => godot_error!("{}", e),
            },
            None => godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE),
        }
    }

    /// Will return network stats of specified handle as a Godot `Dictionary`.
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn get_network_stats(&mut self, handle: u8) -> Dictionary {
        let mut stats = Dictionary::new();
        match &mut self.sess {
            Some(s) => match s.network_stats(handle as PlayerHandle) {
                Ok(n) => {
                    stats.set("send_queue_len", n.send_queue_len as u64);
                    stats.set("ping", n.ping as u64);
                    stats.set("kbps_sent", n.kbps_sent as u64);
                    stats.set("local_frames_behind", n.local_frames_behind);
                    stats.set("remote_frames_behind", n.remote_frames_behind);
                    return stats;
                },
                Err(e) => {
                    godot_error!("{}", e);
                    stats
                }
            },
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE);
                stats
            }
        }
    }

    /// Sets [SessionBuilder::with_input_delay()]
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn set_input_delay(&mut self, delay: u32) {
        self.session_builder = mem::take(&mut self.session_builder).with_input_delay(delay as usize);
    }

    /// Sets [SessionBuilder::with_disconnect_timeout()] converting the u64 to secconds.
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn set_disconnect_timeout(&mut self, secs: u64) {
        self.session_builder = mem::take(&mut self.session_builder).with_disconnect_timeout(std::time::Duration::from_secs(secs));
    }

    /// Sets [SessionBuilder::with_disconnect_notify_delay()] converting the u64 to secconds.
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn set_disconnect_notify_delay(&mut self, secs: u64) {
        self.session_builder = mem::take(&mut self.session_builder).with_disconnect_notify_delay(std::time::Duration::from_secs(secs));
    }

    /// Sets [SessionBuilder::with_sparse_saving_mode()].
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn set_sparse_saving(&mut self, sparse_saving: bool) {
        self.session_builder = mem::take(&mut self.session_builder).with_sparse_saving_mode(sparse_saving);
    }

    /// Disconnects specified player handle.
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn disconnect_player(&mut self, player_handle: u8) {
        match &mut self.sess {
            Some(s) => match s.disconnect_player(player_handle as PlayerHandle) {
                Ok(_) => return,
                Err(e) => godot_error!("{}", e),
            },
            None => godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE),
        }
    }

    /// Returns an `Array` of events which contain usefull information. While you don't have to implement everything, the one thing you should implement is the WaitRecommendation.
    /// For details regarding the events please take a look at [GgrsEvent].
    /// # Example
    /// ```gdscript
    /// var events = ggrs.get_events()
    /// const EVENT_TYPE = "type";
	/// const SKIP_FRAMES = "skip_frames";
    /// const ADDR = "addr";
    ///	for item in events:
	///	    match item[EVENT_TYPE]:
    ///         "WaitRecommendation":
    ///             frames_to_skip += item[SKIP_FRAMES]
    ///         "Synchronizing":
    ///             var addr = item[ADDR];
    ///             ...
    /// ```
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn get_events(&mut self) -> Vec<Dictionary> {
        // TODO: Handle u128 data properly
        let mut result = Vec::new();
        match &mut self.sess {
            Some(s) => {
                for event in s.events() {
                    let mut event_dict = Dictionary::new();
                    match event {
                        GgrsEvent::WaitRecommendation { skip_frames } => {
                            event_dict.set("type", "WaitRecommendation".to_owned().to_variant());
                            event_dict.set("skip_frames", skip_frames.to_variant());
                            result.push(event_dict);
                        }
                        GgrsEvent::NetworkInterrupted {
                            addr,
                            disconnect_timeout,
                        } => {
                            event_dict.set("type", "NetworkInterrupted".to_owned().to_variant());
                            event_dict.set("addr", addr.to_string().to_variant());
                            event_dict.set("disconnect_timeout", Variant::from(disconnect_timeout as u64));
                            result.push(event_dict);
                        }
                        GgrsEvent::NetworkResumed { addr } => {
                            event_dict.set("type", "NetworkResumed".to_owned().to_variant());
                            event_dict.set("addr", addr.to_string().to_variant());
                            result.push(event_dict);
                        }
                        GgrsEvent::Disconnected { addr } => {
                            event_dict.set("type", "Disconnected".to_owned().to_variant());
                            event_dict.set("addr", addr.to_string().to_variant());
                            result.push(event_dict);
                        }
                        GgrsEvent::Synchronized { addr } => {
                            event_dict.set("type", "Synchronized".to_owned().to_variant());
                            event_dict.set("addr", addr.to_string().to_variant());
                            result.push(event_dict);
                        }
                        GgrsEvent::Synchronizing {
                            addr,
                            total,
                            count,
                        } => {
                            event_dict.set("type", "Synchronizing".to_owned().to_variant());
                            event_dict.set("addr", addr.to_string().to_variant());
                            event_dict.set("total", total.to_variant());
                            event_dict.set("count", count.to_variant());
                            result.push(event_dict);
                        }
                        GgrsEvent::DesyncDetected {
                            frame,
                            local_checksum,
                            remote_checksum,
                            addr,
                        } => {
                            event_dict.set("type", "DesyncDetected".to_owned().to_variant());
                            event_dict.set("frame", frame.to_variant());
                            event_dict.set("local_checksum", Variant::from(local_checksum as u64));
                            event_dict.set("remote_checksum", Variant::from(remote_checksum as u64));
                            event_dict.set("addr", addr.to_string().to_variant());
                            result.push(event_dict);
                        }
                    }
                }
            }
            None => godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE),
        };
        return result;
    }

    /// Calls and returns [P2PSession::frames_ahead()].
    /// Will return a 0 if no session was made.
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn get_frames_ahead(&mut self) -> i32 {
        match &mut self.sess {
            Some(s) => s.frames_ahead(),
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE);
                0
            }
        }
    }

    /// Calls and returns [P2PSession::current_frame()].
    /// Will return a 0 if no session was made.
    /// # Errors
    /// - Will print an [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn get_current_frame(&mut self) -> Frame {
        match &mut self.sess {
            Some(s) => s.current_frame(),
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE);
                return 0;
            }
        }
    }

    /// Calls and returns [P2PSession::confirmed_frame()].
    /// Will return a 0 if no session was made.
    /// # Errors
    /// - Will print an [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn get_confirmed_frame(&mut self) -> Frame {
        match &mut self.sess {
            Some(s) => s.confirmed_frame(),
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE);
                return 0;
            }
        }
    }

    //NON-EXPORTED FUNCTIONS
    fn add_player(&mut self, player_type: PlayerType<<GgrsConfig as Config>::Address>) -> PlayerHandle {
        let handle = self.players.len();
        self.players.push((player_type, handle));
        handle
    }
}
