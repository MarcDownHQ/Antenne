//! The plugin's exports, over any [`Http`](crate::host::Http) and [`Storage`](crate::host::Storage).

use crate::catalog;
use crate::model::{Catalog, Config, Error, Preset, ProviderInfo};

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
