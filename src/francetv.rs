//! France Télévisions lives: the k7 descriptor names an untokenised manifest, which hdfauth signs.

use serde_json::Value;

use crate::host::{BROWSER_UA, Http, Request, Storage, encode};
use crate::model::{Error, Playback};

const GEO: &str = "https://geoftv-a.akamaihd.net/ws/edgescape.json";
const K7: &str = "https://k7.ftven.fr/videos/";
const DEFAULT_TOKEN: &str = "https://hdfauth.ftven.fr/esi/TA";
const COUNTRY_KEY: &str = "ftv:country";
const COUNTRY_FOR: u32 = 6 * 3600;
/// The signed master answers for ten minutes (its `hdnea` query); what it names, for six hours (its path token).
pub const SIGNED_FOR: u32 = 300;

pub fn kept_address(si_id: &str) -> String {
    format!("ftv:live:{si_id}")
}

pub fn resolve(http: &impl Http, storage: &impl Storage, si_id: &str) -> Result<Playback, Error> {
    let key = kept_address(si_id);
    if let Some(url) = storage.get(&key) {
        return Ok(Playback::direct(url));
    }
    let country = country(http, storage);
    let k7 = format!("{K7}{si_id}?country_code={country}&os=ios&capabilities=drm&diffusion_mode=tunnel_first&offline=false");
    let response = http.send(&Request::get(k7).header("User-Agent", BROWSER_UA))?;
    let descriptor: Value = serde_json::from_slice(&response.body).unwrap_or(Value::Null);
    if let Some(message) = descriptor["message"].as_str() {
        return Err(Error::BadResponse(format!("France TV: {message}")));
    }
    if !response.ok() {
        return Err(Error::Http(response.status));
    }
    let video = &descriptor["video"];
    if video["drm"] == Value::Bool(true) {
        return Err(Error::BadResponse(format!("France TV {si_id} is DRM-protected")));
    }
    let manifest = video["url"].as_str().ok_or_else(|| Error::BadResponse(format!("France TV {si_id}: no live address")))?;
    if video["format"].as_str() != Some("hls") {
        return Err(Error::BadResponse(format!("France TV {si_id}: not an HLS live")));
    }
    let signed = sign(http, token_endpoint(&video["token"]), manifest)?;
    let _ = storage.set(&key, &signed, Some(SIGNED_FOR), false);
    Ok(Playback::direct(signed))
}

/// The viewer's country as France TV sees it, which k7 wants; France when it can't be told.
fn country(http: &impl Http, storage: &impl Storage) -> String {
    if let Some(country) = storage.get(COUNTRY_KEY) {
        return country;
    }
    let found = http.json(&Request::get(GEO)).ok().and_then(|geo| geo["reponse"]["geo_info"]["country_code"].as_str().map(str::to_string));
    match found {
        Some(country) => {
            let _ = storage.set(COUNTRY_KEY, &country, Some(COUNTRY_FOR), false);
            country
        }
        None => "FR".into(),
    }
}

/// `video.token` is a URL today, and was an object with an `akamai` URL before.
fn token_endpoint(token: &Value) -> &str {
    token.as_str().or_else(|| token["akamai"].as_str()).unwrap_or(DEFAULT_TOKEN)
}

fn sign(http: &impl Http, endpoint: &str, manifest: &str) -> Result<String, Error> {
    let separator = if endpoint.contains('?') { '&' } else { '?' };
    let body = http.text(&Request::get(format!("{endpoint}{separator}url={}", encode(manifest))))?;
    let body = body.trim();
    let signed = if body.starts_with('{') {
        serde_json::from_str::<Value>(body).ok().and_then(|v| v["url"].as_str().map(str::to_string))
    } else {
        Some(body.to_string())
    };
    signed.filter(|u| u.starts_with("https://")).ok_or_else(|| Error::BadResponse(format!("France TV: no signed address for {manifest}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{FakeHttp, FakeStorage, fixture};

    const FRANCE_2: &str = "006194ea-117d-4bcf-94a9-153d999c59ae";
    const MANIFEST: &str = "https://simulcast-b.ftven.fr/simulcast/France_2/hls_fr2/index.m3u8";

    fn k7(si_id: &str, country: &str) -> String {
        format!("{K7}{si_id}?country_code={country}&os=ios&capabilities=drm&diffusion_mode=tunnel_first&offline=false")
    }

    fn hdfauth() -> String {
        format!("https://hdfauth.ftven.fr/esi/TA?format=json&url={}", encode(MANIFEST))
    }

    fn signed() -> String {
        let v: Value = serde_json::from_str(&fixture("ftv-hdfauth.json")).unwrap();
        v["url"].as_str().unwrap().to_string()
    }

    fn france_2() -> FakeHttp {
        FakeHttp::new().on(GEO, 200, &fixture("ftv-geo.json")).on(&k7(FRANCE_2, "FR"), 200, &fixture("ftv-k7-france-2.json")).on(
            &hdfauth(),
            200,
            &fixture("ftv-hdfauth.json"),
        )
    }

    #[test]
    fn signs_the_manifest_k7_names_and_keeps_it() {
        let http = france_2();
        let storage = FakeStorage::new();
        let playback = resolve(&http, &storage, FRANCE_2).unwrap();
        assert_eq!(playback, Playback::direct(signed()));
        assert_eq!(http.sent()[1].headers, vec![("User-Agent".to_string(), BROWSER_UA.to_string())]);
        assert_eq!(storage.value(&kept_address(FRANCE_2)), Some((signed(), Some(SIGNED_FOR), false)));
        assert_eq!(storage.value(COUNTRY_KEY), Some(("FR".into(), Some(COUNTRY_FOR), false)));
    }

    #[test]
    fn a_kept_address_is_played_without_asking() {
        let http = FakeHttp::new();
        let storage = FakeStorage::new().with(&kept_address(FRANCE_2), "https://simulcast-b.ftven.fr/kept/index.m3u8");
        assert_eq!(resolve(&http, &storage, FRANCE_2).unwrap().url, "https://simulcast-b.ftven.fr/kept/index.m3u8");
        assert_eq!(http.urls().len(), 0);
    }

    #[test]
    fn asks_k7_for_the_viewer_country() {
        let http = FakeHttp::new().on(&k7(FRANCE_2, "BE"), 200, &fixture("ftv-k7-france-2.json")).on(
            &hdfauth(),
            200,
            &fixture("ftv-hdfauth.json"),
        );
        let storage = FakeStorage::new().with(COUNTRY_KEY, "BE");
        assert!(resolve(&http, &storage, FRANCE_2).is_ok());
        let unknown = FakeHttp::new().on(GEO, 500, "").on(&k7(FRANCE_2, "FR"), 200, &fixture("ftv-k7-france-2.json")).on(
            &hdfauth(),
            200,
            &fixture("ftv-hdfauth.json"),
        );
        let storage = FakeStorage::new();
        assert!(resolve(&unknown, &storage, FRANCE_2).is_ok());
        assert_eq!(storage.value(COUNTRY_KEY), None);
    }

    #[test]
    fn a_refusal_from_k7_is_told_as_is() {
        let guadeloupe = "1d08fa3f-cf00-481d-a53c-a9d4a46c91ff";
        let http =
            FakeHttp::new().on(GEO, 200, &fixture("ftv-geo.json")).on(&k7(guadeloupe, "FR"), 422, &fixture("ftv-k7-geo-refused.json"));
        let Err(Error::BadResponse(message)) = resolve(&http, &FakeStorage::new(), guadeloupe) else { panic!() };
        assert!(message.starts_with("France TV: Il semblerait que vous soyez dans une zone"), "{message}");
        let down = FakeHttp::new().on(GEO, 200, &fixture("ftv-geo.json")).on(&k7(guadeloupe, "FR"), 503, "<html>");
        assert_eq!(resolve(&down, &FakeStorage::new(), guadeloupe), Err(Error::Http(503)));
    }

    #[test]
    fn refuses_drm_and_anything_but_hls() {
        let with = |video: &str| {
            FakeHttp::new().on(GEO, 200, &fixture("ftv-geo.json")).on(&k7(FRANCE_2, "FR"), 200, &format!(r#"{{"video":{video}}}"#))
        };
        let drm = resolve(&with(r#"{"drm":true,"url":"https://a.test/x.mpd","format":"dash"}"#), &FakeStorage::new(), FRANCE_2);
        assert!(matches!(drm, Err(Error::BadResponse(m)) if m.contains("DRM")));
        let dash = resolve(&with(r#"{"drm":false,"url":"https://a.test/x.mpd","format":"dash"}"#), &FakeStorage::new(), FRANCE_2);
        assert!(matches!(dash, Err(Error::BadResponse(m)) if m.contains("not an HLS")));
        let empty = resolve(&with("{}"), &FakeStorage::new(), FRANCE_2);
        assert!(matches!(empty, Err(Error::BadResponse(m)) if m.contains("no live address")));
    }

    #[test]
    fn takes_either_token_shape() {
        assert_eq!(token_endpoint(&Value::String("https://t.test/a".into())), "https://t.test/a");
        assert_eq!(token_endpoint(&serde_json::json!({"akamai": "https://t.test/b", "drm": "x"})), "https://t.test/b");
        assert_eq!(token_endpoint(&Value::Null), DEFAULT_TOKEN);
    }

    #[test]
    fn signing_takes_json_or_a_bare_address() {
        let plain = format!("{DEFAULT_TOKEN}?url={}", encode(MANIFEST));
        let http = FakeHttp::new().on(&plain, 200, "https://simulcast-b.ftven.fr/tok/index.m3u8\n");
        assert_eq!(sign(&http, DEFAULT_TOKEN, MANIFEST).unwrap(), "https://simulcast-b.ftven.fr/tok/index.m3u8");
        let odd = FakeHttp::new().on(&plain, 200, r#"{"nope":1}"#);
        assert!(matches!(sign(&odd, DEFAULT_TOKEN, MANIFEST), Err(Error::BadResponse(m)) if m.contains("no signed address")));
        let denied = FakeHttp::new().on(&plain, 403, "");
        assert_eq!(sign(&denied, DEFAULT_TOKEN, MANIFEST), Err(Error::Http(403)));
    }

    #[test]
    fn a_signature_that_cannot_be_kept_still_plays() {
        let full = FakeStorage { refuse: true, ..FakeStorage::new() };
        assert!(resolve(&france_2(), &full, FRANCE_2).is_ok());
    }
}
