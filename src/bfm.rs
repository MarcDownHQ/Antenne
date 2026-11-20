//! BFM (NextRadioTV) lives: each live page names its master in the player's `data-video-url`.

use crate::host::{BROWSER_UA, Http, Request, Storage};
use crate::html::attribute;
use crate::model::{Error, Playback};

const SITE: &str = "https://www.bfmtv.com/";
/// The masters are static: a day saves fetching a heavy page on every zap.
const KEPT_FOR: u32 = 24 * 3600;

pub fn kept_address(path: &str) -> String {
    format!("bfm:{path}")
}

pub fn resolve(http: &impl Http, storage: &impl Storage, path: &str) -> Result<Playback, Error> {
    let key = kept_address(path);
    if let Some(url) = storage.get(&key) {
        return Ok(Playback::direct(url));
    }
    // An unknown page is a 404 that still names the national master: only a 2xx page is trusted.
    let page = http.text(&Request::get(format!("{SITE}{path}")).header("User-Agent", BROWSER_UA))?;
    let url = attribute(&page, "bitmovin-player-container", "data-video-url")
        .filter(|u| u.starts_with("https://"))
        .ok_or_else(|| Error::BadResponse(format!("BFM: no player on {SITE}{path}")))?;
    let _ = storage.set(&key, &url, Some(KEPT_FOR), true);
    Ok(Playback::direct(url))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FakeHttp, FakeStorage, fixture};

    const NATIONAL: &str = "https://live-cdn-stream-euw1.bfmtv.bct.nextradiotv.com/master.m3u8";

    #[test]
    fn reads_the_master_from_the_live_page_and_keeps_it() {
        let http = FakeHttp::new().on("https://www.bfmtv.com/en-direct/", 200, &fixture("bfm-en-direct.html"));
        let storage = FakeStorage::new();
        assert_eq!(resolve(&http, &storage, "en-direct/"), Ok(Playback::direct(NATIONAL)));
        assert_eq!(storage.value("bfm:en-direct/"), Some((NATIONAL.into(), Some(KEPT_FOR), true)));
        assert_eq!(resolve(&http, &storage, "en-direct/"), Ok(Playback::direct(NATIONAL)));
        assert_eq!(http.urls().len(), 1);
    }

    #[test]
    fn a_missing_page_is_not_the_national_live() {
        let http = FakeHttp::new().on("https://www.bfmtv.com/paris/en-direct/", 404, &fixture("bfm-en-direct.html"));
        assert_eq!(resolve(&http, &FakeStorage::new(), "paris/en-direct/"), Err(Error::Http(404)));
    }

    #[test]
    fn a_page_without_a_player_says_so() {
        let http = FakeHttp::new().on("https://www.bfmtv.com/lyon/en-direct/", 200, "<html></html>");
        let Err(Error::BadResponse(message)) = resolve(&http, &FakeStorage::new(), "lyon/en-direct/") else { panic!() };
        assert!(message.contains("no player"), "{message}");
    }
}
