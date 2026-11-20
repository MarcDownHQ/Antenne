//! The component boundary: `wit/provider.wit` exports and imports, adapted to the plain Rust of the crate.

#![allow(clippy::too_many_arguments, clippy::same_length_and_capacity, reason = "the code wit-bindgen generates")]

use crate::{host, model};

wit_bindgen::generate!({ path: "wit", world: "provider" });

struct Antenne;

fn error(e: model::Error) -> Error {
    match e {
        model::Error::Http(status) => Error::Http(status),
        model::Error::AuthFailed => Error::AuthFailed,
        model::Error::BadResponse(m) => Error::BadResponse(m),
        model::Error::Network(m) => Error::Network(m),
        model::Error::Setting(m) => Error::Setting(m),
    }
}

struct HostHttp;

impl host::Http for HostHttp {
    fn send(&self, request: &host::Request) -> Result<host::Response, model::Error> {
        use nightglass::provider::http;
        let request = http::Request {
            method: request.method.clone(),
            url: request.url.clone(),
            headers: request.headers.clone(),
            body: request.body.clone(),
            browser: request.browser.map(browser),
        };
        match http::send(&request) {
            Ok(r) => Ok(host::Response { status: r.status, headers: r.headers, body: r.body }),
            Err(e) => Err(from_error(e)),
        }
    }
}

struct HostStorage;

impl host::Storage for HostStorage {
    fn get(&self, key: &str) -> Option<String> {
        nightglass::provider::storage::get(key)
    }

    fn set(&self, key: &str, value: &str, ttl: Option<u32>, persist: bool) -> Result<(), String> {
        nightglass::provider::storage::set(key, value, ttl, persist)
    }

    fn delete(&self, key: &str) {
        nightglass::provider::storage::delete(key);
    }
}

fn browser(b: model::Browser) -> nightglass::provider::types::Browser {
    use nightglass::provider::types::Browser;
    match b {
        model::Browser::Chrome => Browser::Chrome,
        model::Browser::Edge => Browser::Edge,
        model::Browser::Firefox => Browser::Firefox,
        model::Browser::Safari => Browser::Safari,
    }
}

fn from_error(e: Error) -> model::Error {
    match e {
        Error::Http(status) => model::Error::Http(status),
        Error::AuthFailed => model::Error::AuthFailed,
        Error::BadResponse(m) => model::Error::BadResponse(m),
        Error::Network(m) => model::Error::Network(m),
        Error::Setting(m) => model::Error::Setting(m),
    }
}

fn playback(p: model::Playback) -> Playback {
    Playback {
        url: p.url,
        headers: p.headers,
        relay: p.relay.map(|r| nightglass::provider::types::Relay { browser: r.browser.map(browser), filter: r.filter }),
        tracks: Vec::new(),
    }
}

fn channel(c: model::Channel) -> Channel {
    Channel {
        id: c.id,
        name: c.name,
        number: c.number,
        group: c.group,
        logo: c.logo,
        epg_id: None,
        stream: c.stream,
        headers: Vec::new(),
        catchup_days: None,
    }
}

impl Guest for Antenne {
    fn describe() -> ProviderInfo {
        let info: model::ProviderInfo = crate::provider::describe();
        ProviderInfo {
            id: info.id,
            name: info.name,
            fields: Vec::new(),
            resolves_streams: info.resolves_streams,
            presets: info
                .presets
                .into_iter()
                .map(|p| nightglass::provider::types::Preset { label: p.label, name: p.name, config: p.config })
                .collect(),
        }
    }

    fn load_catalog(config: Config) -> Result<Catalog, Error> {
        let catalog = crate::provider::load_catalog(&config).map_err(error)?;
        Ok(Catalog {
            channels: catalog.channels.into_iter().map(channel).collect(),
            movies: Vec::new(),
            series: Vec::new(),
            epg_url: None,
            account: None,
            programmes: Vec::new(),
        })
    }

    fn stream_url(config: Config, channel: Channel) -> Result<Playback, Error> {
        crate::provider::stream_url(&HostHttp, &HostStorage, &config, &channel.id).map(playback).map_err(error)
    }

    fn catchup_url(_config: Config, channel: Channel, _start: i64, _end: i64) -> Result<Playback, Error> {
        crate::provider::catchup_url(&channel.id).map(playback).map_err(error)
    }

    fn episodes(_config: Config, _series: Series) -> Result<Vec<Episode>, Error> {
        crate::provider::episodes().map(|()| Vec::new()).map_err(error)
    }

    fn filter(_config: Config, _url: String, media: Media) -> Result<Media, Error> {
        Ok(media)
    }
}

export!(Antenne);
