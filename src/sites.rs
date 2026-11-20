//! The other players: an API or a page each, read at play time.

use serde_json::Value;

use crate::host::{BROWSER_UA, Http, Request, encode};
use crate::html::{attribute, between, origin};
use crate::model::{Error, Playback};

fn https(url: Option<&str>, what: &str) -> Result<Playback, Error> {
    url.filter(|u| u.starts_with("https://")).map(Playback::direct).ok_or_else(|| Error::BadResponse(format!("{what}: no live address")))
}

/// Kick: the channel info carries a signed IVS address while the channel is live.
pub fn kick(http: &impl Http, slug: &str) -> Result<Playback, Error> {
    let request = Request::get(format!("https://kick.com/api/v2/channels/{slug}/info"))
        .header("Accept", "application/json")
        .header("Referer", "https://player.kick.com/")
        .header("User-Agent", BROWSER_UA);
    let info = http.json(&request)?;
    if info["livestream"].is_null() {
        return Err(Error::BadResponse(format!("Kick {slug} is not live")));
    }
    https(info["playback_url"].as_str(), &format!("Kick {slug}"))
}

/// Equidia: the racing API signs a primary and a backup address per feed.
pub fn equidia(http: &impl Http, name: &str) -> Result<Playback, Error> {
    let request = Request::get(format!("https://api.equidia.fr/api/public/racing/equidia-mobileapp-ios-1/{name}"))
        .header("User-Agent", "Equidia/6036 CFNetwork/1220.1 Darwin/20.3.0")
        .header("Referer", "https://fr.equidia.app/");
    let feed = http.json(&request)?;
    https(feed["primary"].as_str().or_else(|| feed["stream_url_pri"].as_str()), &format!("Equidia {name}"))
}

/// EasyBroadcast: the event API names the stream, and says whether it needs a token.
pub fn easybroadcast(http: &impl Http, tenant: &str, event: &str) -> Result<Playback, Error> {
    let info = http.json(&Request::get(format!("https://{tenant}.player.easybroadcast.io/api/events/{event}")))?;
    let stream = info["stream"]
        .as_str()
        .filter(|u| u.starts_with("https://"))
        .ok_or_else(|| Error::BadResponse(format!("EasyBroadcast {event}: no live address")))?;
    let tokenised = match &info["token_authentication"] {
        Value::Bool(on) => *on,
        Value::String(on) => on.eq_ignore_ascii_case("true"),
        _ => false,
    };
    if !tokenised {
        return Ok(Playback::direct(stream));
    }
    let token = http.text(&Request::get(format!("https://token.easybroadcast.io/all?url={}", encode(stream))))?;
    let separator = if stream.contains('?') { '&' } else { '?' };
    Ok(Playback::direct(format!("{stream}{separator}{}", token.trim().trim_start_matches('?'))))
}

/// `HDR_VISIO` players: the page names a viewer, which names a DASH manifest; the same path serves HLS.
pub fn hdr_visio(http: &impl Http, page: &str) -> Result<Playback, Error> {
    let html = http.text(&Request::get(page).header("User-Agent", BROWSER_UA))?;
    let viewer = attribute(&html, "HDR_VISIO", "data-url").ok_or_else(|| Error::BadResponse(format!("no HDR_VISIO player on {page}")))?;
    let site = origin(page).unwrap_or(page);
    let config = http.json(&Request::get(format!("{site}{viewer}&mode=html")).header("User-Agent", BROWSER_UA))?;
    let manifest = config["files"]["auto"].as_str().map(|u| u.replace("/manifest.mpd", "/playlist.m3u8"));
    https(manifest.as_deref(), page)
}

/// Ultimedia (Digiteka) players: the page carries its configuration as JSON.
pub fn digiteka(http: &impl Http, iframe: &str) -> Result<Playback, Error> {
    let html = http.text(&Request::get(iframe).header("User-Agent", BROWSER_UA))?;
    let config: Value = between(&html, r#"<script id="dtkplayer-conf" type="application/json">"#, "</script>")
        .and_then(|json| serde_json::from_str(json).ok())
        .ok_or_else(|| Error::BadResponse(format!("no Digiteka player on {iframe}")))?;
    https(config["video"]["src"]["live"]["src"].as_str(), iframe)
}

pub fn tv_monaco(http: &impl Http) -> Result<Playback, Error> {
    let request = Request::get("https://videos.tvmonaco.com/api/media/v7/media/le-direct")
        .header("Referer", "https://videos.tvmonaco.com/content/le-direct");
    let media = http.json(&request)?;
    https(media["external_stream"]["url"].as_str(), "TV Monaco")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FakeHttp, fixture};

    #[test]
    fn kick_gives_the_signed_address_of_a_live_channel() {
        let info = "https://kick.com/api/v2/channels/tgplus/info";
        let http = FakeHttp::new().on(info, 200, &fixture("kick-tgplus.json"));
        let playback = kick(&http, "tgplus").unwrap();
        assert!(playback.url.contains(".playback.live-video.net/") && playback.url.contains("token=JWT"), "{}", playback.url);
        assert_eq!(http.sent()[0].headers[1], ("Referer".to_string(), "https://player.kick.com/".to_string()));
        let off = FakeHttp::new().on(info, 200, r#"{"livestream":null,"playback_url":"https://x.test/a.m3u8"}"#);
        assert_eq!(kick(&off, "tgplus"), Err(Error::BadResponse("Kick tgplus is not live".into())));
        let gone = FakeHttp::new().on(info, 404, r#"{"message":"Channel not found."}"#);
        assert_eq!(kick(&gone, "tgplus"), Err(Error::Http(404)));
    }

    #[test]
    fn equidia_takes_the_primary_address_of_either_shape() {
        let api = "https://api.equidia.fr/api/public/racing/equidia-mobileapp-ios-1/";
        let http = FakeHttp::new().on(&format!("{api}equidia-live2"), 200, &fixture("equidia-live2.json"));
        let url = equidia(&http, "equidia-live2").unwrap().url;
        assert!(url.starts_with("https://equidia-live2-p29-cdn.hexaglobe.net/"), "{url}");
        let old = FakeHttp::new().on(&format!("{api}equidia-racing1"), 200, r#"{"stream_url_pri":"https://e.test/1.m3u8"}"#);
        assert_eq!(equidia(&old, "equidia-racing1").unwrap().url, "https://e.test/1.m3u8");
        let none = FakeHttp::new().on(&format!("{api}equidia-racing9"), 200, r#"{"status":"ko"}"#);
        assert_eq!(equidia(&none, "equidia-racing9"), Err(Error::BadResponse("Equidia equidia-racing9: no live address".into())));
    }

    #[test]
    fn easybroadcast_plays_the_event_stream() {
        let api = "https://captaprod.player.easybroadcast.io/api/events/54_tvr_vtucuxt";
        let http = FakeHttp::new().on(api, 200, &fixture("easybroadcast-tvr.json"));
        assert_eq!(
            easybroadcast(&http, "captaprod", "54_tvr_vtucuxt"),
            Ok(Playback::direct("https://cdn.live.easybroadcast.io/abr_corp/54_tvr_vtucuxt/playlist.m3u8"))
        );
        let words = FakeHttp::new().on(api, 200, r#"{"stream":"https://s.test/p.m3u8","token_authentication":"False"}"#);
        assert_eq!(easybroadcast(&words, "captaprod", "54_tvr_vtucuxt").unwrap().url, "https://s.test/p.m3u8");
        let none = FakeHttp::new().on(api, 200, "{}");
        assert!(easybroadcast(&none, "captaprod", "54_tvr_vtucuxt").is_err());
    }

    #[test]
    fn easybroadcast_appends_the_token_when_asked() {
        let api = "https://t.player.easybroadcast.io/api/events/e";
        let token = format!("https://token.easybroadcast.io/all?url={}", encode("https://s.test/p.m3u8"));
        let http =
            FakeHttp::new().on(api, 200, r#"{"stream":"https://s.test/p.m3u8","token_authentication":"True"}"#).on(&token, 200, "?tok=1\n");
        assert_eq!(easybroadcast(&http, "t", "e").unwrap().url, "https://s.test/p.m3u8?tok=1");
        let token = format!("https://token.easybroadcast.io/all?url={}", encode("https://s.test/p.m3u8?a=b"));
        let http =
            FakeHttp::new().on(api, 200, r#"{"stream":"https://s.test/p.m3u8?a=b","token_authentication":true}"#).on(&token, 200, "tok=2");
        assert_eq!(easybroadcast(&http, "t", "e").unwrap().url, "https://s.test/p.m3u8?a=b&tok=2");
    }

    #[test]
    fn hdr_visio_turns_the_dash_manifest_into_hls() {
        let http = FakeHttp::new().on("https://www.viavosges.tv/Direct.html", 200, &fixture("viavosges-direct.html")).on(
            "https://www.viavosges.tv/visionneuse/visio_v9_js.php?key=live1&habillage=2&mode=html",
            200,
            &fixture("viavosges-visio.json"),
        );
        assert_eq!(
            hdr_visio(&http, "https://www.viavosges.tv/Direct.html"),
            Ok(Playback::direct("https://vosgestv.live-kd.com/live/vosgestv/vosgestv/playlist.m3u8"))
        );
    }

    #[test]
    fn hdr_visio_reads_single_quoted_players() {
        let http = FakeHttp::new().on("https://www.tv7.fr/", 200, &fixture("tv7-home.html")).on(
            "https://www.tv7.fr/visionneuse/visio_v9_js.php?key=live1&habillage=0&mode=html",
            200,
            r#"{"files":{"auto":"https:\/\/tv7.live-kd.com\/live\/tv7\/ngrp:tv7_Wifi\/manifest.mpd"}}"#,
        );
        assert_eq!(hdr_visio(&http, "https://www.tv7.fr/").unwrap().url, "https://tv7.live-kd.com/live/tv7/ngrp:tv7_Wifi/playlist.m3u8");
        let bare = FakeHttp::new().on("https://www.tv7.fr/", 200, "<html></html>");
        assert!(matches!(hdr_visio(&bare, "https://www.tv7.fr/"), Err(Error::BadResponse(m)) if m.contains("no HDR_VISIO")));
    }

    #[test]
    fn digiteka_reads_the_player_configuration() {
        let iframe = "https://www.ultimedia.com/deliver/generic/iframe/mdtk/01521514/zone/1/showtitle/1/src/qvrlxxf";
        let http = FakeHttp::new().on(iframe, 200, &fixture("ultimedia-iframe.html"));
        assert_eq!(digiteka(&http, iframe), Ok(Playback::direct("https://live-20minutestv.digiteka.com/1961167769/index.m3u8")));
        let old = FakeHttp::new().on(iframe, 200, "<script>DtkPlayer.init({})</script>");
        assert!(matches!(digiteka(&old, iframe), Err(Error::BadResponse(m)) if m.contains("no Digiteka")));
    }

    #[test]
    fn tv_monaco_reads_the_external_stream() {
        let http = FakeHttp::new().on("https://videos.tvmonaco.com/api/media/v7/media/le-direct", 200, &fixture("tvmonaco-le-direct.json"));
        let url = tv_monaco(&http).unwrap().url;
        assert!(url.starts_with("https://production-fast-mcrtv.content.okast.tv/channels/") && url.ends_with("/master.m3u8"), "{url}");
    }
}
