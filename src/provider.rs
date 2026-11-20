//! The plugin's exports, over any [`Http`](crate::host::Http) and [`Storage`](crate::host::Storage).

use crate::catalog::{self, Source};
use crate::dailymotion;
use crate::host::{Http, Storage};
use crate::model::{Catalog, Config, Error, Playback, Preset, ProviderInfo};

pub const ID: &str = "antenne";
pub const NAME: &str = "Antenne";

pub fn describe() -> ProviderInfo {
    ProviderInfo {
        id: ID.into(),
        name: NAME.into(),
        resolves_streams: true,
        presets: vec![Preset { label: "French live TV".into(), name: NAME.into(), config: Vec::new() }],
    }
}

pub fn load_catalog(_config: &Config) -> Result<Catalog, Error> {
    Ok(catalog::load())
}

/// The live of the channel with this id, resolved now: most addresses are signed and short-lived.
pub fn stream_url(http: &impl Http, storage: &impl Storage, _config: &Config, channel: &str) -> Result<Playback, Error> {
    let entry = catalog::find(channel).ok_or_else(|| Error::BadResponse(format!("unknown channel: {channel}")))?;
    match entry.source {
        Source::Dailymotion { video, embedder } => dailymotion::resolve(http, storage, video, embedder),
        _ => Err(Error::BadResponse(format!("{} cannot be played yet", entry.name))),
    }
}

pub fn catchup_url(channel: &str) -> Result<Playback, Error> {
    Err(Error::BadResponse(format!("no replay on Antenne yet ({channel})")))
}

pub fn episodes() -> Result<(), Error> {
    Err(Error::BadResponse("Antenne has live channels only".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FakeHttp, FakeStorage, fixture};

    #[test]
    fn describes_a_provider_that_resolves_at_play_time() {
        let info = describe();
        assert_eq!(info.id, "antenne");
        assert!(info.resolves_streams);
        assert_eq!(info.presets.len(), 1);
        assert_eq!(info.presets[0].config, Vec::new());
    }

    #[test]
    fn the_catalog_needs_no_setting() {
        assert_eq!(load_catalog(&Vec::new()).unwrap().channels.len(), catalog::CHANNELS.len());
    }

    #[test]
    fn resolves_a_channel_by_id() {
        let meta = "https://www.dailymotion.com/player/metadata/video/x3b68jn?embedder=";
        let http =
            FakeHttp::new().on(meta, 200, &fixture("dm-metadata-cnews.json")).on("https://cdndirector.dailymotion.com/*", 200, "#EXTM3U");
        let playback = stream_url(&http, &FakeStorage::new(), &Vec::new(), "cnews").unwrap();
        assert!(playback.url.starts_with("https://cdndirector.dailymotion.com/cdn/live/video/x3b68jn.m3u8?"));
    }

    #[test]
    fn unknown_channels_and_replays_are_clear_errors() {
        let none = FakeHttp::new();
        assert_eq!(stream_url(&none, &FakeStorage::new(), &Vec::new(), "tf1"), Err(Error::BadResponse("unknown channel: tf1".into())));
        assert!(matches!(catchup_url("cnews"), Err(Error::BadResponse(m)) if m.contains("no replay")));
        assert!(episodes().is_err());
        assert!(matches!(stream_url(&none, &FakeStorage::new(), &Vec::new(), "france-2"), Err(Error::BadResponse(m)) if m.contains("yet")));
    }
}
