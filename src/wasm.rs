//! The component boundary: `wit/provider.wit` exports and imports, adapted to the plain Rust of the crate.

#![allow(clippy::too_many_arguments, clippy::same_length_and_capacity, reason = "the code wit-bindgen generates")]

use crate::model;

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

fn not_yet() -> Error {
    Error::BadResponse("not implemented yet".into())
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

    fn stream_url(_config: Config, _channel: Channel) -> Result<Playback, Error> {
        Err(not_yet())
    }

    fn catchup_url(_config: Config, _channel: Channel, _start: i64, _end: i64) -> Result<Playback, Error> {
        Err(not_yet())
    }

    fn episodes(_config: Config, _series: Series) -> Result<Vec<Episode>, Error> {
        Err(not_yet())
    }

    fn filter(_config: Config, _url: String, media: Media) -> Result<Media, Error> {
        Ok(media)
    }
}

export!(Antenne);
