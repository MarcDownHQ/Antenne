//! The plugin's exports, over any [`Http`](crate::host::Http) and [`Storage`](crate::host::Storage).

use crate::catalog::{self, Source};
use crate::host::{Http, Storage};
use crate::model::{Catalog, Config, Error, LicenseRequest, Media, Playback, Preset, ProviderInfo};
use crate::{bfm, dailymotion, francetv, hls, m6plus, sites};

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
        Source::FranceTv { si_id } => francetv::resolve(http, storage, si_id),
        Source::Bfm { path } => bfm::resolve(http, storage, path),
        Source::Kick { slug } => sites::kick(http, slug),
        Source::Equidia { name } => sites::equidia(http, name),
        Source::EasyBroadcast { tenant, event } => sites::easybroadcast(http, tenant, event),
        Source::HdrVisio { page } => sites::hdr_visio(http, page),
        Source::Digiteka { iframe } => sites::digiteka(http, iframe),
        Source::TvMonaco => sites::tv_monaco(http),
        Source::M6plus { code } => m6plus::resolve(http, storage, code, now()),
        Source::Static { url } => Ok(Playback::direct(url)),
    }
}

/// Every answer of a relayed playback whose relay filters, before the player gets it.
pub fn filter(storage: &impl Storage, _config: &Config, url: &str, media: Media) -> Media {
    hls::filter(storage, url, media)
}

pub fn catchup_url(channel: &str) -> Result<Playback, Error> {
    Err(Error::BadResponse(format!("no replay on Antenne yet ({channel})")))
}

pub fn episodes() -> Result<(), Error> {
    Err(Error::BadResponse("Antenne has live channels only".into()))
}

pub fn video_url(video: &str) -> Result<Playback, Error> {
    Err(Error::BadResponse(format!("Antenne has live channels only ({video})")))
}

/// The license server's answer to a message of the host's CDM, for a playback whose `drm` Antenne gave.
pub fn license(http: &impl Http, storage: &impl Storage, _config: &Config, request: &LicenseRequest) -> Result<Vec<u8>, Error> {
    m6plus::license(http, storage, request, now())
}

fn now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
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
    fn france_tv_channels_resolve_through_k7() {
        let storage =
            FakeStorage::new().with(&crate::francetv::kept_address("006194ea-117d-4bcf-94a9-153d999c59ae"), "https://a.test/f2.m3u8");
        assert_eq!(stream_url(&FakeHttp::new(), &storage, &Vec::new(), "france-2"), Ok(Playback::direct("https://a.test/f2.m3u8")));
    }

    #[test]
    fn every_other_source_resolves_through_its_player() {
        let http = FakeHttp::new()
            .on("https://kick.com/*", 200, r#"{"livestream":{},"playback_url":"https://k.test/a.m3u8"}"#)
            .on("https://api.equidia.fr/*", 200, r#"{"primary":"https://e.test/a.m3u8"}"#)
            .on("https://captaprod.player.easybroadcast.io/*", 200, r#"{"stream":"https://eb.test/a.m3u8"}"#)
            .on("https://www.tv7.fr/", 200, &fixture("tv7-home.html"))
            .on("https://www.tv7.fr/visionneuse/*", 200, r#"{"files":{"auto":"https://kd.test/manifest.mpd"}}"#)
            .on("https://www.ultimedia.com/*", 200, &fixture("ultimedia-iframe.html"))
            .on("https://videos.tvmonaco.com/*", 200, r#"{"external_stream":{"url":"https://m.test/a.m3u8"}}"#);
        let storage = FakeStorage::new().with("bfm:lyon/en-direct/", "https://b.test/lyon.m3u8");
        let url = |id: &str| stream_url(&http, &storage, &Vec::new(), id).unwrap().url;
        assert_eq!(url("bfm-lyon"), "https://b.test/lyon.m3u8");
        assert_eq!(url("tg-plus"), "https://k.test/a.m3u8");
        assert_eq!(url("equidia"), "https://e.test/a.m3u8");
        assert_eq!(url("tvr"), "https://eb.test/a.m3u8");
        assert_eq!(url("tv7-colmar"), "https://kd.test/playlist.m3u8");
        assert_eq!(url("20-minutes-tv-idf"), "https://live-20minutestv.digiteka.com/1961167769/index.m3u8");
        assert_eq!(url("tv-monaco"), "https://m.test/a.m3u8");
        assert_eq!(url("kto"), "https://livekto.akamaized.net/hls/live/20000018/ktotv/master.m3u8");
    }

    #[test]
    fn m6_plus_channels_resolve_to_a_widevine_live_and_get_their_license() {
        use crate::m6plus::tests::{LIVE_M6, UPFRONT_M6, paired};
        let live = r#"{"M6":[{"live":{"assets":[{"type":"delta_dashcenc_h264","video_quality":"hd","full_physical_path":"https://m6.test/a.mpd"}]}}]}"#;
        let http = FakeHttp::new().on(LIVE_M6, 200, live).on(UPFRONT_M6, 200, r#"{"token":"up"}"#).on(
            m6plus::LICENSE,
            200,
            r#"{"status":"OK","license":"CAU="}"#,
        );
        let storage = paired().with(m6plus::TOKEN, "box");
        let playback = stream_url(&http, &storage, &Vec::new(), "m6").unwrap();
        assert_eq!(playback.url, "https://m6.test/a.mpd");
        let drm = playback.drm.unwrap();
        let request =
            LicenseRequest { kind: crate::model::LicenseKind::Certificate, challenge: vec![8, 4], url: playback.url, context: drm.context };
        assert_eq!(license(&http, &storage, &Vec::new(), &request), Ok(vec![8, 5]));
    }

    #[test]
    fn filters_relayed_answers() {
        let media = Media { content_type: None, body: b"segment".to_vec() };
        assert_eq!(filter(&FakeStorage::new(), &Vec::new(), "https://a.test/1.ts", media.clone()), media);
    }

    #[test]
    fn unknown_channels_and_replays_are_clear_errors() {
        let none = FakeHttp::new();
        assert_eq!(stream_url(&none, &FakeStorage::new(), &Vec::new(), "tf1"), Err(Error::BadResponse("unknown channel: tf1".into())));
        assert!(matches!(catchup_url("cnews"), Err(Error::BadResponse(m)) if m.contains("no replay")));
        assert!(episodes().is_err());
        assert!(matches!(video_url("jt-20h"), Err(Error::BadResponse(m)) if m.contains("live channels only")));
        let request = LicenseRequest {
            kind: crate::model::LicenseKind::Request,
            challenge: vec![8, 1],
            url: "https://a.test/a.mpd".into(),
            context: String::new(),
        };
        let refused = license(&none, &FakeStorage::new(), &Vec::new(), &request);
        assert!(matches!(refused, Err(Error::BadResponse(m)) if m.contains("not an M6+ playback")));
    }
}
