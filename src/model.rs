//! The records of `wit/provider.wit` the plugin produces, as plain Rust.

pub type Config = Vec<(String, String)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Browser {
    Chrome,
    Edge,
    Firefox,
    Safari,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Http(u16),
    AuthFailed,
    BadResponse(String),
    Network(String),
    Setting(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Preset {
    pub label: String,
    pub name: String,
    pub config: Config,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub resolves_streams: bool,
    pub presets: Vec<Preset>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Channel {
    pub id: String,
    pub name: String,
    pub number: Option<i64>,
    pub group: Option<String>,
    pub logo: Option<String>,
    pub stream: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Catalog {
    pub channels: Vec<Channel>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Relay {
    pub browser: Option<Browser>,
    pub filter: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Playback {
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub relay: Option<Relay>,
}

impl Playback {
    /// Played as is: no headers, no relay.
    pub fn direct(url: impl Into<String>) -> Self {
        Playback { url: url.into(), headers: Vec::new(), relay: None }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Media {
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}
