# Overview

One crate, `antenne`, built two ways:

- for `wasm32-wasip2` as a `cdylib`: the component Nightglass loads, `antenne.wasm`;
- natively as an `rlib`: what the tests, the coverage and the smoke check run.

Only `src/wasm.rs` is wasm-specific (`#[cfg(target_family = "wasm")]`). It runs `wit_bindgen::generate!` on
`wit/provider.wit`, implements the generated `Guest` trait by calling `provider`, and adapts the host imports to two
traits: `HostHttp` implements `Http` with `http.send`, `HostStorage` implements `Storage` with `storage`. Everything
else is plain Rust over those traits, so it builds and is tested on the host.

## Modules

| Module | Role |
| --- | --- |
| `model` | The WIT records Antenne produces, as plain Rust: `ProviderInfo`, `Channel`, `Catalog`, `Playback`, `Relay`, `Media`, `Error` |
| `host` | `Request`, `Response`, the `Http` and `Storage` traits, query escaping, the browser User-Agent |
| `provider` | The exports: `describe`, `load_catalog`, `stream_url` (dispatch by source), `filter`, the replay errors |
| `catalog` | The compiled-in channel table and its `Source` for each channel |
| `dailymotion` | Metadata, master checks, retries, the ladder memory |
| `hls` | Master parsing: the one-encoder filter, the tallest variant |
| `francetv` | Country, k7, hdfauth, the kept signature |
| `bfm` | The BFM live pages |
| `sites` | Kick, Equidia, EasyBroadcast, `HDR_VISIO`, Ultimedia, TV Monaco |
| `html` | Reading an attribute or a JSON block out of a page |
| `testing` | `FakeHttp` (answers by URL, in turn), `FakeStorage`, fixtures (tests only) |

## A play, end to end

1. Nightglass calls `stream-url` with the channel. Antenne finds the entry by `channel.id` (the catalog's `stream`
   is a stand-in, `https://antenne.invalid/<id>`, because the host drops channels whose stream is not a network
   address).
2. The entry's `Source` picks the resolver, which asks the service and returns a `Playback`: the address, and for
   Dailymotion a relay with a Chrome handshake and the filter on.
3. Through a relay, every answer of the stream comes back through `filter`, which only rewrites Dailymotion masters.

## What is kept between calls

Each call runs in a fresh instance, so anything worth keeping goes through `Storage`. Losing it costs requests,
never a play: a failed `set` is ignored.

| Key | Value | Lifetime | Why |
| --- | --- | --- | --- |
| `dm:master:<video>` | The master `stream-url` checked | 120 s, memory | `filter` prefers it when the relay's copy has a shorter ladder |
| `dm:ladder:<video>` | Tallest variant seen, in lines | 1 day (1 h when the ladder fell short), on disk | When to stop asking for a fuller ladder |
| `ftv:live:<si_id>` | The signed France TV master | 300 s, memory | Zapping back skips k7 and hdfauth |
| `ftv:country` | The country France TV sees | 6 h, memory | k7 wants it on every call |
| `bfm:<path>` | The master named by a BFM page | 1 day, on disk | The masters are static; the pages are heavy |

## Sources

- [src/lib.rs](../../src/lib.rs)
- [src/wasm.rs](../../src/wasm.rs)
- [src/model.rs](../../src/model.rs)
- [src/host.rs](../../src/host.rs)
- [src/provider.rs](../../src/provider.rs)
- [src/testing.rs](../../src/testing.rs)
