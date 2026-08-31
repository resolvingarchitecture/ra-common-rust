//! Ports `ra.common.messaging.CommandMessage`.

use serde::{Deserialize, Serialize};

/// A command for a service to execute. Ports `CommandMessage.Command`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Command {
    Start,
    Shutdown,
    GracefullyShutdown,
    Restart,
    Pause,
    Unpause,
    NetState,
    Report,
    RegisterStateChangeListener,
    UnregisterStateChangeListener,
}

/// A message telling a service which [`Command`] to run.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommandMessage {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub error_messages: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<Command>,
}

impl CommandMessage {
    /// A message carrying `command`.
    pub fn new(command: Command) -> Self {
        CommandMessage {
            error_messages: Vec::new(),
            command: Some(command),
        }
    }
}
