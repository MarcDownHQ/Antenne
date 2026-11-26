//! Antenne: French live TV channels for Nightglass, as a WebAssembly component.
//!
//! Everything but [`wasm`] is plain Rust that builds and is tested natively; `wasm` adapts the host imports and
//! exports of `wit/provider.wit` to it.

pub mod bfm;
pub mod catalog;
pub mod dailymotion;
pub mod francetv;
mod hls;
pub mod host;
mod html;
pub mod m6plus;
pub mod model;
pub mod provider;
pub mod sites;

#[cfg(test)]
mod testing;

#[cfg(target_family = "wasm")]
mod wasm;
