use serde::{Deserialize, Serialize};

/// Messages flowing gateway → game.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GameMsg {
    /// character_id is Some when the gateway already knows who this socket belongs to
    /// (i.e. the game rebooted but the player stayed connected).
    Connect    { client_id: u32, addr: String, character_id: Option<String> },
    Input      { client_id: u32, line: String },
    Disconnect { client_id: u32 },
}

/// Messages flowing game → gateway.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GatewayMsg {
    Output        { client_id: u32, text: String },
    Broadcast     { text: String },
    /// Game requests the gateway close this client's TCP connection.
    Disconnect    { client_id: u32 },
    /// Game tells the gateway which character owns this socket, so the gateway
    /// can relay it automatically if the game reboots.
    Authenticated { client_id: u32, character_id: String },
    /// Disconnect every client with a message; gateway stays running.
    DisconnectAll { message: String },
    /// Shut the gateway process down entirely (used for full server shutdown).
    Shutdown,
}
