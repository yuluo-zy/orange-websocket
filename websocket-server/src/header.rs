use websocket_core::sec_header::{WebSocketAccept, WebSocketKey};

pub enum Header {
    Accept(WebSocketAccept),
    Extensions(Vec<Extension>),
    Key(WebSocketKey),
    Origin(String),
    Protocol(Vec<String>),
    Version(String)
}

pub struct Extension {
    pub name: String,
    pub params: Vec<Parameter>
}

pub struct Parameter {
    pub name: String,
    pub value: Option<String>,
}
