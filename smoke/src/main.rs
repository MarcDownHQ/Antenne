//! Live check: `cargo run -p antenne-smoke -- [CHANNEL...]` resolves each channel against the real services, as the
//! app would, then fetches its master (through `filter` when the playback asks for it), one variant and one audio
//! rendition. With the clients the Nightglass host uses: reqwest, or wreq with a browser's handshake when asked.

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Write;
use std::process::ExitCode;

use antenne::host::{Http, Request, Response, Storage};
use antenne::model::{Browser, Error, Media, Playback};
use antenne::provider;
use reqwest::Url;
use wreq_util::Emulation;

struct Client {
    runtime: tokio::runtime::Runtime,
    plain: reqwest::Client,
    chrome: wreq::Client,
    firefox: wreq::Client,
}

impl Client {
    fn new() -> Self {
        let browser =
            |emulation| wreq::Client::builder().emulation(emulation).redirect(wreq::redirect::Policy::limited(10)).build().expect("wreq");
        Client {
            runtime: tokio::runtime::Builder::new_multi_thread().enable_all().build().expect("runtime"),
            plain: reqwest::Client::builder().user_agent("Nightglass/smoke").build().expect("reqwest"),
            chrome: browser(Emulation::Chrome149),
            firefox: browser(Emulation::Firefox151),
        }
    }

    /// The answer and where it came from after redirects.
    fn fetch(&self, request: &Request) -> Result<(Response, String), Error> {
        let network = |e: &dyn std::fmt::Display| Error::Network(e.to_string());
        self.runtime.block_on(async {
            match request.browser {
                None => {
                    let mut builder = self.plain.get(&request.url);
                    for (k, v) in &request.headers {
                        builder = builder.header(k, v);
                    }
                    let r = builder.send().await.map_err(|e| network(&e))?;
                    let (status, url) = (r.status().as_u16(), r.url().to_string());
                    let headers = r.headers().iter().filter_map(|(k, v)| Some((k.to_string(), v.to_str().ok()?.to_string()))).collect();
                    let body = r.bytes().await.map_err(|e| network(&e))?.to_vec();
                    Ok((Response { status, headers, body }, url))
                }
                Some(browser) => {
                    let client = if browser == Browser::Firefox { &self.firefox } else { &self.chrome };
                    let mut builder = client.get(&request.url);
                    for (k, v) in &request.headers {
                        builder = builder.header(k.as_str(), v.as_str());
                    }
                    let r = builder.send().await.map_err(|e| network(&e))?;
                    let (status, url) = (r.status().as_u16(), r.uri().to_string());
                    let headers = r.headers().iter().filter_map(|(k, v)| Some((k.to_string(), v.to_str().ok()?.to_string()))).collect();
                    let body = r.bytes().await.map_err(|e| network(&e))?.to_vec();
                    Ok((Response { status, headers, body }, url))
                }
            }
        })
    }
}

impl Http for Client {
    fn send(&self, request: &Request) -> Result<Response, Error> {
        self.fetch(request).map(|(r, _)| r)
    }
}

#[derive(Default)]
struct Memory(RefCell<HashMap<String, String>>);

impl Storage for Memory {
    fn get(&self, key: &str) -> Option<String> {
        self.0.borrow().get(key).cloned()
    }

    fn set(&self, key: &str, value: &str, _ttl: Option<u32>, _persist: bool) -> Result<(), String> {
        self.0.borrow_mut().insert(key.into(), value.into());
        Ok(())
    }

    fn delete(&self, key: &str) {
        self.0.borrow_mut().remove(key);
    }
}

fn uris(playlist: &str) -> Vec<&str> {
    playlist.lines().filter(|l| !l.is_empty() && !l.starts_with('#')).collect()
}

fn quoted_uri(line: &str) -> Option<&str> {
    let start = line.find("URI=\"")? + 5;
    Some(&line[start..start + line[start..].find('"')?])
}

/// Fetches what the player would, as the relay would.
fn check(client: &Client, storage: &Memory, playback: &Playback) -> Result<String, String> {
    let browser = playback.relay.as_ref().and_then(|r| r.browser);
    let get = |url: &str| {
        let mut request = Request::get(url);
        request.headers.clone_from(&playback.headers);
        request.browser = browser;
        client.fetch(&request).map_err(|e| format!("{url}: {e:?}"))
    };
    let (master, at) = get(&playback.url)?;
    if !master.ok() {
        return Err(format!("master answered {}", master.status));
    }
    let mut body = master.body;
    if playback.relay.as_ref().is_some_and(|r| r.filter) {
        body = provider::filter(storage, &Vec::new(), &at, Media { content_type: None, body }).body;
    }
    let text = String::from_utf8_lossy(&body).into_owned();
    if !text.starts_with("#EXTM3U") {
        return Err("the master is not a playlist".into());
    }
    let base = Url::parse(&at).map_err(|e| e.to_string())?;
    let variants = text.lines().filter(|l| l.starts_with("#EXT-X-STREAM-INF:")).count();
    let audio: Vec<&str> = text.lines().filter(|l| l.starts_with("#EXT-X-MEDIA:TYPE=AUDIO")).collect();
    let muxed = text.lines().any(|l| l.starts_with("#EXT-X-STREAM-INF:") && l.contains("mp4a"));
    let mut report = format!("{variants} variants, {} audio renditions, codecs list audio: {muxed}", audio.len());
    let playlist = |uri: &str, what: &str| -> Result<usize, String> {
        let url = base.join(uri).map_err(|e| e.to_string())?;
        let (r, _) = get(url.as_str())?;
        let t = String::from_utf8_lossy(&r.body).into_owned();
        if !r.ok() || !t.starts_with("#EXTM3U") {
            return Err(format!("{what} playlist answered {}", r.status));
        }
        Ok(uris(&t).len())
    };
    if let Some(first) = uris(&text).first() {
        let _ = write!(report, "; first variant: {} segments", playlist(first, "variant")?);
    }
    if let Some(uri) = audio.first().and_then(|l| quoted_uri(l)) {
        let _ = write!(report, "; first audio rendition: {} segments", playlist(uri, "audio")?);
    }
    if variants == 0 || (audio.is_empty() && !muxed) {
        return Err(format!("no variants or no audio: {report}"));
    }
    Ok(report)
}

fn main() -> ExitCode {
    let mut ids: Vec<String> = std::env::args().skip(1).collect();
    if ids.is_empty() {
        ids = ["cnews", "france-2", "bfmtv"].map(String::from).to_vec();
    }
    let (client, storage) = (Client::new(), Memory::default());
    let mut failed = false;
    for id in &ids {
        let outcome = provider::stream_url(&client, &storage, &Vec::new(), id)
            .map_err(|e| format!("{e:?}"))
            .and_then(|p| check(&client, &storage, &p));
        match outcome {
            Ok(report) => println!("ok   {id}: {report}"),
            Err(e) => {
                failed = true;
                println!("FAIL {id}: {e}");
            }
        }
    }
    if failed { ExitCode::FAILURE } else { ExitCode::SUCCESS }
}
