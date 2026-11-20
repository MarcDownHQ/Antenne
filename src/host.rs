//! What the plugin needs from its host: `http` and `storage` of `wit/provider.wit`, as traits, so the logic runs
//! natively against fakes and recorded answers.

use crate::model::{Browser, Error};

pub const BROWSER_UA: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/138.0.0.0 Safari/537.36";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
    pub browser: Option<Browser>,
}

impl Request {
    pub fn get(url: impl Into<String>) -> Self {
        Request { method: "GET".into(), url: url.into(), headers: Vec::new(), body: None, browser: None }
    }

    #[must_use]
    pub fn header(mut self, name: &str, value: &str) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    #[must_use]
    pub fn browser(mut self, browser: Browser) -> Self {
        self.browser = Some(browser);
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Response {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Response {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v.as_str())
    }

    pub fn ok(&self) -> bool {
        (200..300).contains(&self.status)
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

/// `http.send`: any status is a response; only a failed connection is an error.
pub trait Http {
    fn send(&self, request: &Request) -> Result<Response, Error>;

    /// The body of a 2xx answer, any other status as [`Error::Http`].
    fn text(&self, request: &Request) -> Result<String, Error> {
        let response = self.send(request)?;
        if response.ok() { Ok(response.text()) } else { Err(Error::Http(response.status)) }
    }

    fn json(&self, request: &Request) -> Result<serde_json::Value, Error> {
        let text = self.text(request)?;
        serde_json::from_str(&text).map_err(|e| Error::BadResponse(format!("{}: {e}", request.url)))
    }
}

/// `storage`: values kept between calls, each of which runs in a fresh instance.
pub trait Storage {
    fn get(&self, key: &str) -> Option<String>;
    fn set(&self, key: &str, value: &str, ttl: Option<u32>, persist: bool) -> Result<(), String>;
    fn delete(&self, key: &str);
}

/// `application/x-www-form-urlencoded` style escaping for a query value.
pub fn encode(value: &str) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(value.len() * 3);
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(char::from(b));
        } else {
            let _ = write!(out, "%{b:02X}");
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::FakeHttp;

    #[test]
    fn requests_carry_headers_and_a_browser() {
        let r = Request::get("https://a.test/").header("Referer", "https://b.test/").browser(Browser::Firefox);
        assert_eq!(r.method, "GET");
        assert_eq!(r.headers, vec![("Referer".to_string(), "https://b.test/".to_string())]);
        assert_eq!(r.browser, Some(Browser::Firefox));
        assert_eq!(r.body, None);
    }

    #[test]
    fn headers_match_whatever_their_case() {
        let r = Response { status: 200, headers: vec![("Content-Type".into(), "text/html".into())], body: Vec::new() };
        assert_eq!(r.header("content-type"), Some("text/html"));
        assert_eq!(r.header("location"), None);
    }

    #[test]
    fn text_and_json_need_a_2xx() {
        let http = FakeHttp::new().on("https://a.test/ok", 200, r#"{"a":1}"#).on("https://a.test/gone", 404, "nope");
        assert_eq!(http.json(&Request::get("https://a.test/ok")).unwrap()["a"], 1);
        assert_eq!(http.text(&Request::get("https://a.test/gone")), Err(Error::Http(404)));
        let bad = http.json(&Request::get("https://a.test/gone"));
        assert_eq!(bad, Err(Error::Http(404)));
    }

    #[test]
    fn json_that_does_not_parse_is_a_bad_response() {
        let http = FakeHttp::new().on("https://a.test/", 200, "<html>");
        assert!(matches!(http.json(&Request::get("https://a.test/")), Err(Error::BadResponse(m)) if m.starts_with("https://a.test/")));
    }

    #[test]
    fn encodes_query_values() {
        assert_eq!(encode("https://a.test/x y?b=1&c"), "https%3A%2F%2Fa.test%2Fx%20y%3Fb%3D1%26c");
        assert_eq!(encode("A-z_0.9~"), "A-z_0.9~");
    }
}
