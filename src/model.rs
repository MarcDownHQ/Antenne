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
    ActionRequired(Prompt),
}

/// Something the user must do outside the app, such as entering `code` at `url`; the app asks again after `retry_in`
/// seconds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prompt {
    pub message: String,
    pub url: Option<String>,
    pub code: Option<String>,
    pub retry_in: Option<u32>,
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
    pub protected: bool,
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
    pub drm: Option<Drm>,
}

impl Playback {
    /// Played as is: no headers, no relay.
    pub fn direct(url: impl Into<String>) -> Self {
        Playback { url: url.into(), headers: Vec::new(), relay: None, drm: None }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Manifest {
    Dash,
    Hls,
}

/// A Widevine stream, the only key system the host plays. `context` comes back with every license request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Drm {
    pub manifest: Manifest,
    pub certificate: Option<Vec<u8>>,
    pub pssh: Option<Vec<u8>>,
    pub context: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LicenseKind {
    Request,
    Renewal,
    Release,
    Certificate,
}

/// A message of the host's CDM for the license server, with the `url` and `drm.context` of its playback.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LicenseRequest {
    pub kind: LicenseKind,
    pub challenge: Vec<u8>,
    pub url: String,
    pub context: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Media {
    pub content_type: Option<String>,
    pub body: Vec<u8>,
}
