use crate::{PopupDefinition, PopupResult};
use serde::{Deserialize, Serialize};

/// Messages sent from server (Cloudflare DO) to client (local daemon)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Request client to show a popup
    ShowPopup {
        /// Unique identifier for this popup request
        id: String,
        /// Popup structure definition
        definition: PopupDefinition,
        /// Timeout in milliseconds before auto-canceling
        timeout_ms: u64,
    },
    /// Request client to close a popup (if still open)
    ClosePopup {
        /// Identifier of popup to close
        id: String,
    },
    /// Keepalive ping
    Ping,
}

/// Messages sent from client (local daemon) to server (Cloudflare DO)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Client ready to receive popup requests
    Ready {
        /// Optional device name for identification
        #[serde(skip_serializing_if = "Option::is_none")]
        device_name: Option<String>,
    },
    /// Result from a completed popup interaction
    Result {
        /// Identifier matching the show_popup request
        id: String,
        /// User interaction result
        result: PopupResult,
    },
    /// Keepalive pong response
    Pong,
}

impl ServerMessage {
    /// Create a show_popup message
    pub fn show_popup(id: String, definition: PopupDefinition, timeout_ms: u64) -> Self {
        ServerMessage::ShowPopup {
            id,
            definition,
            timeout_ms,
        }
    }

    /// Create a close_popup message
    pub fn close_popup(id: String) -> Self {
        ServerMessage::ClosePopup { id }
    }

    /// Create a ping message
    pub fn ping() -> Self {
        ServerMessage::Ping
    }
}

impl ClientMessage {
    /// Create a ready message
    pub fn ready(device_name: Option<String>) -> Self {
        ClientMessage::Ready { device_name }
    }

    /// Create a result message
    pub fn result(id: String, result: PopupResult) -> Self {
        ClientMessage::Result { id, result }
    }

    /// Create a pong message
    pub fn pong() -> Self {
        ClientMessage::Pong
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Element;

    #[test]
    fn test_server_message_serialization() {
        let popup_def = PopupDefinition {
            title: Some("Test".to_string()),
            elements: vec![Element::Text {
                text: "Hello".to_string(),
                id: None,
                when: None,
            }],
        };

        let msg = ServerMessage::show_popup("test-123".to_string(), popup_def, 30000);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains(r#""type":"show_popup"#));
        assert!(json.contains(r#""id":"test-123"#));
        assert!(json.contains(r#""timeout_ms":30000"#));

        // Deserialize back
        let deserialized: ServerMessage = serde_json::from_str(&json).unwrap();
        match deserialized {
            ServerMessage::ShowPopup {
                id,
                definition,
                timeout_ms,
            } => {
                assert_eq!(id, "test-123");
                assert_eq!(definition.title, Some("Test".to_string()));
                assert_eq!(timeout_ms, 30000);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_client_message_serialization() {
        let msg = ClientMessage::ready(Some("laptop-1".to_string()));
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains(r#""type":"ready"#));
        assert!(json.contains(r#""device_name":"laptop-1"#));

        // Ready with no device name
        let msg2 = ClientMessage::ready(None);
        let json2 = serde_json::to_string(&msg2).unwrap();
        assert!(!json2.contains("device_name"));
    }

    #[test]
    fn test_ping_pong_serialization() {
        let ping = ServerMessage::ping();
        let ping_json = serde_json::to_string(&ping).unwrap();
        assert_eq!(ping_json, r#"{"type":"ping"}"#);

        let pong = ClientMessage::pong();
        let pong_json = serde_json::to_string(&pong).unwrap();
        assert_eq!(pong_json, r#"{"type":"pong"}"#);
    }

    #[test]
    fn test_result_message_serialization() {
        let result = PopupResult::Cancelled;
        let msg = ClientMessage::result("popup-456".to_string(), result);
        let json = serde_json::to_string(&msg).unwrap();

        assert!(json.contains(r#""type":"result"#));
        assert!(json.contains(r#""id":"popup-456"#));
    }
}
