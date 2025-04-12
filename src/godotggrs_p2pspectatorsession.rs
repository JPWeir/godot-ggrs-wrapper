use crate::*;
use ggrs::{GgrsEvent, SessionBuilder, SessionState, SpectatorSession, UdpNonBlockingSocket};
use std::mem;

/// A Godot implementation of [`P2PSpectatorSession`]
#[derive(GodotClass)]
#[class(base=Node)]
pub struct GodotGgrsP2PSpectatorSession {
    base: Base<Node>,
    session_builder: SessionBuilder<GgrsConfig>,
    sess: Option<SpectatorSession<GgrsConfig>>,
    callback_node: Option<Gd<Node>>
}

#[godot_api]
impl INode for GodotGgrsP2PSpectatorSession {
    fn init(base: Base<Node>) -> Self {
        GodotGgrsP2PSpectatorSession {
            base,
            session_builder: SessionBuilder::new(),
            sess: None,
            callback_node: None
        }
    }
}

#[godot_api]
impl GodotGgrsP2PSpectatorSession {
    //EXPORTED FUNCTIONS
    #[func]
    fn _ready(&self) {
        godot_print!("GodotGGRSP2PSpectatorSession _ready() called.");
    }

    /// Returns true if connection has been established with remote players and is ready to start taking inputs via [Self::advance_frame()]
    #[func]
    pub fn is_running(&mut self) -> bool {
        match &mut self.sess {
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

    /// Starts the [P2PSpectatorSession]
    /// # Errors
    /// - Will print a [GgrsError] error if no session is made.
    /// - Will print a [std::io::Error] if the local_port cannot be bound.
    /// - Will panic if the address string could not be converted to an [std::net::SocketAddr]
    #[func]
    pub fn start_session(&mut self, host_addr: String, local_port: u16) {
        let remote_addr: std::net::SocketAddr = host_addr.parse().unwrap();
        match UdpNonBlockingSocket::bind_to_port(local_port) {
            Ok(socket) => {
                let new_session = mem::take(&mut self.session_builder).start_spectator_session(remote_addr, socket);
                self.sess = Some(new_session);
            },
            Err(e) => {
                godot_error!("{}", e);
            }
        };
    }

    /// Sets the callback node that will be called when using [Self::advance_frame()]
    #[func]
    pub fn set_callback_node(&mut self, callback: Gd<Node>) {
        self.callback_node = Some(callback);
    }

    /// This function will advance the frame using the inputs given as a parameter (currently an int in Godot)
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

    /// Returns [P2PSpectatorSession::frames_behind_host()]
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn get_frames_behind_host(&mut self) -> u32 {
        match &mut self.sess {
            Some(s) => return s.frames_behind_host() as u32,
            None => {
                godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE);
                return 0;
            }
        }
    }

    /// Sets [SessionBuilder::with_catchup_speed()]
    /// Cannot be called after starting the session
    /// # Errors
    /// - Will print a [GgrsError] error if the catchup speed cannot be set
    #[func]
    pub fn set_catchup_speed(&mut self, desired_catchup_speed: u32) {
        let builder = mem::take(&mut self.session_builder);
        match builder.with_catchup_speed(desired_catchup_speed as usize) {
            Ok(new_builder) => self.session_builder = new_builder,
            Err(e) => godot_error!("{}", e),
        }
    }

    /// Sets [SessionBuilder::with_max_frames_behind()]
    /// Cannot be called after starting the session
    /// # Errors
    /// - Will print a [GgrsError] error if the max frames behind cannot be
    #[func]
    pub fn set_max_frames_behind(&mut self, desired_value: u32) {
        let builder = mem::take(&mut self.session_builder);
        match builder.with_max_frames_behind(desired_value as usize) {
            Ok(new_builder) => self.session_builder = new_builder,
            Err(e) => godot_error!("{}", e),
        }
    }

    /// Calls [P2PSpectatorSession::poll_remote_clients()]
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn poll_remote_clients(&mut self, ) {
        match &mut self.sess {
            Some(s) => s.poll_remote_clients(),
            None => godot_error!("{}", ERR_MESSAGE_NO_SESSION_MADE),
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

    /// Prints out network stats of host address
    /// # Errors
    /// - Will print a [ERR_MESSAGE_NO_SESSION_MADE] error if a session has not been made
    #[func]
    pub fn print_network_stats(&mut self) {
        match &mut self.sess {
            Some(s) => match s.network_stats() {
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
    pub fn get_network_stats(&mut self) -> Dictionary {
        let mut stats = Dictionary::new();
        match &mut self.sess {
            Some(s) => match s.network_stats() {
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
}
