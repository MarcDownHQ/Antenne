//! M6+ (M6, W9, Gulli, 6ter): Antenne pairs itself with the viewer's account as an M6+ TV would, by a code the
//! viewer enters on m6.fr, then keeps the device and the account, never a password.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, KeyInit, Mac};
use serde_json::Value;
use sha1::Sha1;

use crate::host::{Http, Request, Storage, encode};
use crate::model::{Error, Prompt};

pub const PAIRING_URL: &str = "https://www.m6.fr/device-pairing";
const USERS: &str = "https://6play-users.6play.fr";
const AUTH: &str = "https://front-auth.6cloud.fr";
const SALT: &str = "da39a3ee5e6b4b0d3255bfef95601890afd80709";
pub const CUSTOMER: &str = "m6web";
/// How often the M6+ TV apps ask whether the code was entered, and how long they show one.
const POLL: u32 = 3;
const CODE_LIFETIME: u32 = 300;
/// Taken off a token's expiry, so a token is never sent in its last minute.
const MARGIN: u64 = 60;

pub const DEVICE: &str = "m6:device";
pub const ACCOUNT: &str = "m6:account";
pub const CODE: &str = "m6:code";
pub const TOKEN: &str = "m6:token";

/// A paired device: its id, the account it is paired with, and a token for that account.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub device: String,
    pub account: String,
    pub token: String,
}

/// The kept session, renewing its token when it has expired. Without a pairing, the viewer is asked to enter a code,
/// and each call until then asks the service whether they did.
pub fn session(http: &impl Http, storage: &impl Storage, now: u64) -> Result<Session, Error> {
    let device = device(storage)?;
    let account = match storage.get(ACCOUNT) {
        Some(account) => account,
        None => pair(http, storage, &device, now)?,
    };
    let token = match storage.get(TOKEN) {
        Some(token) => token,
        None => renew(http, storage, &device, &account, now)?,
    };
    Ok(Session { device, account, token })
}

/// Drops the kept token, which a service refused.
pub fn forget_token(storage: &impl Storage) {
    storage.delete(TOKEN);
}

fn device(storage: &impl Storage) -> Result<String, Error> {
    if let Some(device) = storage.get(DEVICE) {
        return Ok(device);
    }
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|e| Error::BadResponse(format!("no random source: {e}")))?;
    let device = uuid(bytes);
    let _ = storage.set(DEVICE, &device, None, true);
    Ok(device)
}

fn uuid(mut bytes: [u8; 16]) -> String {
    bytes[6] = bytes[6] & 0x0f | 0x40;
    bytes[8] = bytes[8] & 0x3f | 0x80;
    let h = hex(&bytes);
    format!("{}-{}-{}-{}-{}", &h[..8], &h[8..12], &h[12..16], &h[16..20], &h[20..])
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes.iter().fold(String::with_capacity(bytes.len() * 2), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    })
}

fn box_id(device: &str) -> String {
    format!("deviceid-{device}")
}

/// The pairing the kept code led to, or the prompt to enter it. A code the service no longer knows is replaced.
fn pair(http: &impl Http, storage: &impl Storage, device: &str, now: u64) -> Result<String, Error> {
    if let Some(code) = storage.get(CODE) {
        let path = format!("/v2/platforms/smarttv/boxTypes/smarttv/boxIds/{}/tokens/{code}/status", box_id(device));
        let response = http.send(&signed(Request::get(format!("{USERS}{path}")), &path, &[], device, now))?;
        match response.status {
            404 => storage.delete(CODE),
            200 => {
                let status: Value = serde_json::from_slice(&response.body).map_err(|e| Error::BadResponse(format!("M6+ pairing: {e}")))?;
                if status["status"] == "paired"
                    && let Some(account) = status["uid"].as_str().filter(|a| !a.is_empty())
                {
                    if storage.set(ACCOUNT, account, None, true).is_ok() {
                        storage.delete(CODE);
                    }
                    return Ok(account.into());
                }
                return Err(prompt(&code));
            }
            status => return Err(Error::Http(status)),
        }
    }
    let path = "/v2/platforms/smarttv/box";
    let id = box_id(device);
    let body = serde_json::json!({ "boxType": "smarttv", "boxId": id }).to_string().into_bytes();
    let request = Request::post(format!("{USERS}{path}"), body).header("Content-Type", "application/json");
    let answer = http.json(&signed(request, path, &[("boxId", &id), ("boxType", "smarttv")], device, now))?;
    let code = answer["token"].as_str().ok_or_else(|| Error::BadResponse("M6+ gave no pairing code".into()))?;
    let _ = storage.set(CODE, code, Some(CODE_LIFETIME), true);
    Err(prompt(code))
}

fn prompt(code: &str) -> Error {
    Error::ActionRequired(Prompt {
        message: "Sign in to M6+ with this code".into(),
        url: Some(PAIRING_URL.into()),
        code: Some(code.into()),
        retry_in: Some(POLL),
    })
}

/// A new token for the account, kept until a minute before it expires. The service still hands one out once the
/// viewer has removed the pairing, so the pairing is checked first; without it, the viewer is asked to pair again.
fn renew(http: &impl Http, storage: &impl Storage, device: &str, account: &str, now: u64) -> Result<String, Error> {
    let path = format!("/v2/platforms/smarttv/boxTypes/smarttv/boxIds/{}/users", box_id(device));
    let users = http.json(&signed(Request::get(format!("{USERS}{path}")), &path, &[], device, now))?;
    if !users.as_array().is_some_and(|users| users.iter().any(|u| u["uid"] == account)) {
        for key in [ACCOUNT, CODE, TOKEN] {
            storage.delete(key);
        }
        return pair(http, storage, device, now).and(Err(Error::AuthFailed));
    }
    let path = "/v2/platforms/smarttv/getJwt";
    let request = signed(Request::get(format!("{AUTH}{path}")), path, &[], device, now)
        .header("x-auth-box-id", &box_id(device))
        .header("x-auth-box-type", "smarttv")
        .header("X-Auth-gigya-uid", account);
    let answer = http.json(&request)?;
    let token = answer["token"].as_str().ok_or_else(|| Error::BadResponse("M6+ gave no token".into()))?;
    let ttl = expiry(token).map_or(0, |exp| exp.saturating_sub(now + MARGIN));
    if ttl > 0 {
        let _ = storage.set(TOKEN, token, Some(u32::try_from(ttl).unwrap_or(u32::MAX)), false);
    }
    Ok(token.into())
}

fn expiry(jwt: &str) -> Option<u64> {
    let payload = URL_SAFE_NO_PAD.decode(jwt.split('.').nth(1)?.trim_end_matches('=')).ok()?;
    serde_json::from_slice::<Value>(&payload).ok()?["exp"].as_u64()
}

/// The headers every M6+ TV call carries: the device, and an HMAC of the path, the body's fields and the minute.
fn signed(request: Request, path: &str, fields: &[(&str, &str)], device: &str, now: u64) -> Request {
    let mut fields = fields.to_vec();
    fields.sort_unstable();
    let body = fields.iter().map(|(k, v)| format!("{}={}", encode(k), encode(v))).collect::<Vec<_>>().join("&");
    let mut mac = Hmac::<Sha1>::new_from_slice(SALT.as_bytes()).expect("HMAC takes any key");
    mac.update(format!("{path}{body}{}", now / 60).as_bytes());
    request
        .header("X-Customer-Name", CUSTOMER)
        .header("x-auth-device-id", device)
        .header("X-Auth-Token", &hex(&mac.finalize().into_bytes()))
        .header("X-Auth-Token-Timestamp", &now.to_string())
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::host::Response;
    use crate::testing::{FakeHttp, FakeStorage};

    pub const NOW: u64 = 1_764_150_000;
    pub const STATUS: &str = "https://6play-users.6play.fr/v2/platforms/smarttv/boxTypes/smarttv/boxIds/deviceid-d1/tokens/ABCDE/status";
    pub const USERS_OF_D1: &str = "https://6play-users.6play.fr/v2/platforms/smarttv/boxTypes/smarttv/boxIds/deviceid-d1/users";
    pub const START: &str = "https://6play-users.6play.fr/v2/platforms/smarttv/box";
    pub const JWT: &str = "https://front-auth.6cloud.fr/v2/platforms/smarttv/getJwt";
    /// A token whose payload is `{"exp": NOW + 1800, "iat": NOW}`.
    pub const BOX_TOKEN: &str = "h.eyJleHAiOiAxNzY0MTUxODAwLCAiaWF0IjogMTc2NDE1MDAwMH0.s";

    pub fn paired() -> FakeStorage {
        FakeStorage::new().with(DEVICE, "d1").with(ACCOUNT, "u1")
    }

    pub fn header<'a>(request: &'a Request, name: &str) -> Option<&'a str> {
        request.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }

    fn code_prompt(code: &str) -> Error {
        prompt(code)
    }

    #[test]
    fn a_first_play_makes_a_device_and_shows_a_code() {
        let http = FakeHttp::new().on(START, 200, r#"{"token":"ABCDE"}"#);
        let storage = FakeStorage::new();
        let Err(Error::ActionRequired(p)) = session(&http, &storage, NOW) else { panic!("a prompt") };
        assert_eq!((p.url.as_deref(), p.code.as_deref(), p.retry_in), (Some(PAIRING_URL), Some("ABCDE"), Some(3)));
        let (device, ttl, persist) = storage.value(DEVICE).unwrap();
        assert_eq!((device.len(), &device[14..15], ttl, persist), (36, "4", None, true));
        assert_eq!(storage.value(CODE), Some(("ABCDE".into(), Some(300), true)));
        let start = &http.sent()[0];
        assert_eq!(start.method, "POST");
        let body: Value = serde_json::from_slice(start.body.as_deref().unwrap()).unwrap();
        assert_eq!(body, serde_json::json!({ "boxType": "smarttv", "boxId": format!("deviceid-{device}") }));
        assert_eq!(header(start, "x-auth-device-id"), Some(device.as_str()));
        assert_eq!(header(start, "X-Customer-Name"), Some("m6web"));
    }

    #[test]
    fn calls_are_signed_over_path_fields_and_minute() {
        let http = FakeHttp::new().on(START, 200, r#"{"token":"ABCDE"}"#);
        let _ = session(&http, &FakeStorage::new().with(DEVICE, "d1"), NOW);
        let start = &http.sent()[0];
        assert_eq!(header(start, "X-Auth-Token"), Some("89fdc7259aa77be9decc6c0dd22322f13d964e0f"));
        assert_eq!(header(start, "X-Auth-Token-Timestamp"), Some("1764150000"));
        let request = signed(Request::get(JWT), "/v2/platforms/smarttv/getJwt", &[], "d1", NOW + 59);
        assert_eq!(header(&request, "X-Auth-Token"), Some("f344bdb7d23021dbbcf00a9fcc4ecd5b2d120241"));
    }

    #[test]
    fn a_code_is_asked_about_until_it_is_entered() {
        let storage = FakeStorage::new().with(DEVICE, "d1").with(CODE, "ABCDE");
        let http = FakeHttp::new().on(STATUS, 200, r#"{"status":"waiting","uid":null}"#);
        assert_eq!(session(&http, &storage, NOW), Err(code_prompt("ABCDE")));
        assert_eq!(http.urls(), [STATUS]);
        assert_eq!(header(&http.sent()[0], "x-auth-device-id"), Some("d1"));
    }

    #[test]
    fn an_expired_code_is_replaced() {
        let storage = FakeStorage::new().with(DEVICE, "d1").with(CODE, "ABCDE");
        let http = FakeHttp::new().on(STATUS, 404, "Token expired or not found").on(START, 200, r#"{"token":"FGHIJ"}"#);
        assert_eq!(session(&http, &storage, NOW), Err(code_prompt("FGHIJ")));
        assert_eq!(storage.get(CODE).as_deref(), Some("FGHIJ"));
    }

    #[test]
    fn an_entered_code_pairs_the_account_and_gets_a_token() {
        let storage = FakeStorage::new().with(DEVICE, "d1").with(CODE, "ABCDE");
        let http = FakeHttp::new()
            .on(STATUS, 200, r#"{"status":"paired","uid":"u1"}"#)
            .on(USERS_OF_D1, 200, r#"[{"uid":"u1","email":"a@b.test"}]"#)
            .on(JWT, 200, &format!(r#"{{"token":"{BOX_TOKEN}"}}"#));
        let session = session(&http, &storage, NOW).unwrap();
        assert_eq!(session, Session { device: "d1".into(), account: "u1".into(), token: BOX_TOKEN.into() });
        assert_eq!(storage.value(ACCOUNT), Some(("u1".into(), None, true)));
        assert_eq!(storage.get(CODE), None);
        assert_eq!(storage.value(TOKEN), Some((BOX_TOKEN.into(), Some(1740), false)));
        let jwt = &http.sent()[2];
        assert_eq!(header(jwt, "x-auth-box-id"), Some("deviceid-d1"));
        assert_eq!(header(jwt, "x-auth-box-type"), Some("smarttv"));
        assert_eq!(header(jwt, "X-Auth-gigya-uid"), Some("u1"));
        assert!(header(jwt, "X-Auth-Token").is_some());
    }

    #[test]
    fn a_kept_token_costs_no_request() {
        let http = FakeHttp::new();
        let session = session(&http, &paired().with(TOKEN, "t"), NOW).unwrap();
        assert_eq!(session.token, "t");
        assert_eq!(http.urls(), Vec::<String>::new());
    }

    #[test]
    fn a_removed_pairing_asks_for_a_new_code() {
        let storage = paired().with(CODE, "OLDER");
        let http = FakeHttp::new().on(USERS_OF_D1, 200, "[]").on(START, 200, r#"{"token":"FGHIJ"}"#);
        assert_eq!(session(&http, &storage, NOW), Err(code_prompt("FGHIJ")));
        assert_eq!(storage.get(ACCOUNT), None);
        assert_eq!(http.urls(), [USERS_OF_D1, START]);
    }

    #[test]
    fn a_token_without_expiry_is_used_once() {
        let http = FakeHttp::new().on(USERS_OF_D1, 200, r#"[{"uid":"u1"}]"#).on(JWT, 200, r#"{"token":"opaque"}"#);
        let storage = paired();
        assert_eq!(session(&http, &storage, NOW).unwrap().token, "opaque");
        assert_eq!(storage.get(TOKEN), None);
        let storage = storage.with(TOKEN, "t");
        forget_token(&storage);
        assert_eq!(storage.get(TOKEN), None);
    }

    #[test]
    fn a_pairing_kept_nowhere_is_asked_about_again() {
        let storage = FakeStorage { refuse: true, ..FakeStorage::new().with(DEVICE, "d1").with(CODE, "ABCDE") };
        let http = FakeHttp::new().on(STATUS, 200, r#"{"status":"paired","uid":"u1"}"#).on(USERS_OF_D1, 200, r#"[{"uid":"u1"}]"#).on(
            JWT,
            200,
            &format!(r#"{{"token":"{BOX_TOKEN}"}}"#),
        );
        assert_eq!(session(&http, &storage, NOW).unwrap().account, "u1");
        assert_eq!(storage.get(CODE).as_deref(), Some("ABCDE"));
    }

    #[test]
    fn service_failures_are_errors() {
        let storage = FakeStorage::new().with(DEVICE, "d1").with(CODE, "ABCDE");
        assert_eq!(session(&FakeHttp::new().on(STATUS, 500, ""), &storage, NOW), Err(Error::Http(500)));
        assert!(matches!(session(&FakeHttp::new().on(STATUS, 200, "<html>"), &storage, NOW), Err(Error::BadResponse(_))));
        let fresh = FakeStorage::new().with(DEVICE, "d1");
        assert!(matches!(session(&FakeHttp::new().on(START, 200, "{}"), &fresh, NOW), Err(Error::BadResponse(m)) if m.contains("code")));
        let http = FakeHttp::new().on(USERS_OF_D1, 200, r#"[{"uid":"u1"}]"#).on(JWT, 200, "{}");
        assert!(matches!(session(&http, &paired(), NOW), Err(Error::BadResponse(m)) if m.contains("token")));
        let http = FakeHttp::new().answer(USERS_OF_D1, Ok(Response { status: 498, headers: Vec::new(), body: Vec::new() }));
        assert_eq!(session(&http, &paired(), NOW), Err(Error::Http(498)));
    }

    #[test]
    fn device_ids_are_version_4_uuids() {
        assert_eq!(uuid([0xff; 16]), "ffffffff-ffff-4fff-bfff-ffffffffffff");
        assert_eq!(expiry("a.!!.c"), None);
    }
}
