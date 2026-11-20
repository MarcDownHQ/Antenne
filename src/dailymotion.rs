//! Dailymotion lives: the player metadata gives a signed HLS master, which the app relays with a browser handshake.

use serde_json::Value;

use crate::hls::max_height;
use crate::host::{Http, Request, Storage, encode};
use crate::model::{Browser, Error, Playback, Relay};

const METADATA: &str = "https://www.dailymotion.com/player/metadata/video/";
const MASTERS: &str = "https://cdndirector.dailymotion.com/cdn/live/video/";
const ATTEMPTS: usize = 5;
/// The handshake the master needs: over the app's own client the master answers 403 (`E005`) nearly every time.
pub const BROWSER: Browser = Browser::Chrome;
const KEPT_FOR: u32 = 120;
const FULL_HEIGHT: u32 = 720;
const LADDERS: usize = 3;
const LADDER_FOR: u32 = 24 * 3600;
const SHORT_LADDER_FOR: u32 = 3600;

pub fn kept_master(video: &str) -> String {
    format!("dm:master:{video}")
}

pub fn kept_ladder(video: &str) -> String {
    format!("dm:ladder:{video}")
}

/// The video id of a master address, for the masters Dailymotion signs.
pub fn master_video(url: &str) -> Option<&str> {
    let rest = url.strip_prefix(MASTERS)?;
    rest.split(['.', '?']).next().filter(|id| !id.is_empty())
}

/// The whole master: its audio is in separate renditions on most channels, so no single variant is enough.
///
/// Some signed addresses list a cut-down ladder (480p at most) for as long as they live, so new ones are asked for
/// until the ladder reaches the height last seen for this video, within a few tries.
pub fn resolve(http: &impl Http, storage: &impl Storage, video: &str, embedder: Option<&str>) -> Result<Playback, Error> {
    let wanted = storage.get(&kept_ladder(video)).and_then(|h| h.parse().ok()).unwrap_or(FULL_HEIGHT);
    let mut best: Option<(u32, String, String)> = None;
    let mut ladders = 0;
    let mut last = Error::BadResponse(format!("Dailymotion {video}: no answer"));
    for _ in 0..ATTEMPTS {
        let master = master_url(http, video, embedder)?;
        match http.send(&Request::get(&master).browser(BROWSER)) {
            Ok(response) if response.ok() => {
                let body = response.text();
                let height = max_height(&body);
                if best.as_ref().is_none_or(|(h, _, _)| height > *h) {
                    best = Some((height, master, body));
                }
                ladders += 1;
                if height >= wanted || ladders == LADDERS {
                    break;
                }
            }
            Ok(response) => last = Error::Http(response.status),
            Err(e) => last = e,
        }
    }
    let Some((height, master, body)) = best else { return Err(last) };
    let _ = storage.set(&kept_master(video), &body, Some(KEPT_FOR), false);
    let ladder_for = if height >= wanted { LADDER_FOR } else { SHORT_LADDER_FOR };
    let _ = storage.set(&kept_ladder(video), &height.to_string(), Some(ladder_for), true);
    Ok(Playback { url: master, headers: Vec::new(), relay: Some(Relay { browser: Some(BROWSER), filter: true }) })
}

fn master_url(http: &impl Http, video: &str, embedder: Option<&str>) -> Result<String, Error> {
    let url = format!("{METADATA}{video}?embedder={}", encode(embedder.unwrap_or_default()));
    let metadata = http.json(&Request::get(url).header("Referer", "https://www.dailymotion.com/").browser(BROWSER))?;
    if let Some(error) = metadata.get("error") {
        let title = error["title"].as_str().unwrap_or("unavailable");
        return Err(Error::BadResponse(match error["raw_message"].as_str() {
            Some(message) => format!("Dailymotion: {title} ({message})"),
            None => format!("Dailymotion: {title}"),
        }));
    }
    if metadata["protected_delivery"] == Value::Bool(true) {
        return Err(Error::BadResponse(format!("Dailymotion {video} is DRM-protected")));
    }
    metadata["qualities"]["auto"][0]["url"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| Error::BadResponse(format!("Dailymotion {video}: no live address in the metadata")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::Response;
    use crate::testing::{FakeHttp, FakeStorage, fixture};

    const CNEWS_META: &str = "https://www.dailymotion.com/player/metadata/video/x3b68jn?embedder=";

    fn cnews_master() -> String {
        let meta: Value = serde_json::from_str(&fixture("dm-metadata-cnews.json")).unwrap();
        meta["qualities"]["auto"][0]["url"].as_str().unwrap().to_string()
    }

    fn denied() -> Response {
        Response { status: 403, headers: vec![("x-error-code".into(), "E005".into())], body: b"<html>".to_vec() }
    }

    #[test]
    fn hands_over_the_whole_master_relayed_with_a_browser_handshake() {
        let http = FakeHttp::new().on(CNEWS_META, 200, &fixture("dm-metadata-cnews.json")).on(
            &cnews_master(),
            200,
            &fixture("dm-master-cnews.m3u8"),
        );
        let storage = FakeStorage::new();
        let playback = resolve(&http, &storage, "x3b68jn", None).unwrap();
        assert_eq!(playback.url, cnews_master());
        assert_eq!(playback.relay, Some(Relay { browser: Some(Browser::Chrome), filter: true }));
        assert_eq!(playback.headers, Vec::new());
        let sent = http.sent();
        assert_eq!(sent[0].headers, vec![("Referer".to_string(), "https://www.dailymotion.com/".to_string())]);
        assert!(sent.iter().all(|r| r.browser == Some(Browser::Chrome)));
        assert_eq!(sent[1].headers, Vec::new());
        let (kept, ttl, persist) = storage.value("dm:master:x3b68jn").unwrap();
        assert!(kept.starts_with("#EXTM3U"));
        assert_eq!((ttl, persist), (Some(120), false));
    }

    #[test]
    fn a_denied_master_means_new_metadata_then_a_new_try() {
        let master = cnews_master();
        let http = FakeHttp::new()
            .on(CNEWS_META, 200, &fixture("dm-metadata-cnews.json"))
            .answer(&master, Ok(denied()))
            .answer(&master, Ok(denied()))
            .on(&master, 200, &ladder(1080));
        let playback = resolve(&http, &FakeStorage::new(), "x3b68jn", None).unwrap();
        assert_eq!(playback.url, master);
        assert_eq!(http.urls().iter().filter(|u| u.as_str() == CNEWS_META).count(), 3);
    }

    #[test]
    fn gives_up_after_five_tries_with_the_last_failure() {
        let master = cnews_master();
        let http = FakeHttp::new().on(CNEWS_META, 200, &fixture("dm-metadata-cnews.json")).answer(&master, Ok(denied()));
        assert_eq!(resolve(&http, &FakeStorage::new(), "x3b68jn", None), Err(Error::Http(403)));
        assert_eq!(http.urls().len(), 10);
        let unreachable = FakeHttp::new().on(CNEWS_META, 200, &fixture("dm-metadata-cnews.json"));
        assert!(matches!(resolve(&unreachable, &FakeStorage::new(), "x3b68jn", None), Err(Error::Network(_))));
    }

    fn ladder(height: u32) -> String {
        format!("#EXTM3U\n#EXT-X-STREAM-INF:BANDWIDTH={height},RESOLUTION=1x{height}\nhttps://l.test/x3b68jn/{height}.m3u8\n")
    }

    #[test]
    fn a_cut_down_ladder_is_asked_again() {
        let master = cnews_master();
        let http = FakeHttp::new().on(CNEWS_META, 200, &fixture("dm-metadata-cnews.json")).on(&master, 200, &ladder(477)).on(
            &master,
            200,
            &ladder(1080),
        );
        let storage = FakeStorage::new();
        assert!(resolve(&http, &storage, "x3b68jn", None).is_ok());
        assert_eq!(http.urls().len(), 4);
        assert!(storage.get("dm:master:x3b68jn").unwrap().contains("/1080.m3u8"));
        assert_eq!(storage.value("dm:ladder:x3b68jn"), Some(("1080".into(), Some(LADDER_FOR), true)));
    }

    #[test]
    fn a_short_ladder_is_taken_after_three_tries_and_remembered_briefly() {
        let master = cnews_master();
        let http = FakeHttp::new().on(CNEWS_META, 200, &fixture("dm-metadata-cnews.json")).on(&master, 200, &ladder(288)).on(
            &master,
            200,
            &ladder(477),
        );
        let storage = FakeStorage::new();
        assert!(resolve(&http, &storage, "x3b68jn", None).is_ok());
        assert_eq!(http.urls().len(), 6);
        assert!(storage.get("dm:master:x3b68jn").unwrap().contains("/477.m3u8"));
        assert_eq!(storage.value("dm:ladder:x3b68jn"), Some(("477".into(), Some(SHORT_LADDER_FOR), true)));
        let again = FakeHttp::new().on(CNEWS_META, 200, &fixture("dm-metadata-cnews.json")).on(&master, 200, &ladder(477));
        assert!(resolve(&again, &storage, "x3b68jn", None).is_ok());
        assert_eq!(again.urls().len(), 2);
        assert_eq!(storage.value("dm:ladder:x3b68jn"), Some(("477".into(), Some(LADDER_FOR), true)));
    }

    #[test]
    fn a_master_that_cannot_be_kept_still_plays() {
        let http = FakeHttp::new().on(CNEWS_META, 200, &fixture("dm-metadata-cnews.json")).on(&cnews_master(), 200, "#EXTM3U");
        let full = FakeStorage { refuse: true, ..FakeStorage::new() };
        assert!(resolve(&http, &full, "x3b68jn", None).is_ok());
    }

    #[test]
    fn passes_the_embedder_page() {
        let meta = "https://www.dailymotion.com/player/metadata/video/x9jhhyc?embedder=https%3A%2F%2Ft18.fr%2F";
        let http = FakeHttp::new().on(meta, 200, &fixture("dm-metadata-cnews.json")).on(&cnews_master(), 200, "#EXTM3U");
        assert!(resolve(&http, &FakeStorage::new(), "x9jhhyc", Some("https://t18.fr/")).is_ok());
    }

    #[test]
    fn an_offline_live_says_so() {
        let meta = "https://www.dailymotion.com/player/metadata/video/x9u4hka?embedder=";
        let http = FakeHttp::new().on(meta, 200, &fixture("dm-metadata-offline.json"));
        let Err(Error::BadResponse(message)) = resolve(&http, &FakeStorage::new(), "x9u4hka", None) else { panic!() };
        assert!(message.starts_with("Dailymotion: Live déconnecté ("), "{message}");
        assert_eq!(http.urls().len(), 1);
        let bare = FakeHttp::new().on(meta, 200, r#"{"error":{}}"#);
        assert_eq!(resolve(&bare, &FakeStorage::new(), "x9u4hka", None), Err(Error::BadResponse("Dailymotion: unavailable".into())));
    }

    #[test]
    fn refuses_protected_or_empty_metadata() {
        let http = FakeHttp::new().on(CNEWS_META, 200, r#"{"protected_delivery":true}"#);
        assert!(matches!(resolve(&http, &FakeStorage::new(), "x3b68jn", None), Err(Error::BadResponse(m)) if m.contains("DRM")));
        let http = FakeHttp::new().on(CNEWS_META, 200, r#"{"qualities":{}}"#);
        assert!(
            matches!(resolve(&http, &FakeStorage::new(), "x3b68jn", None), Err(Error::BadResponse(m)) if m.contains("no live address"))
        );
    }

    #[test]
    fn masters_name_their_video() {
        assert_eq!(master_video(&cnews_master()), Some("x3b68jn"));
        assert_eq!(master_video("https://cdndirector.dailymotion.com/cdn/live/video/.m3u8"), None);
        assert_eq!(master_video("https://live.cf.dmcdn.net/x.m3u8"), None);
    }
}
