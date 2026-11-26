<p align="center"><img src="icon.svg" alt="Antenne" width="128"></p>

# Antenne

French live TV for [Nightglass](https://github.com/MarcDownHQ/nightglass).

Antenne is a Nightglass plugin: a WebAssembly component that adds a source of 99 French live channels. The national
public channels and the France 3 editions come from France Télévisions, CNews, L'Équipe, T18 and the radios with a
video live from Dailymotion, BFM TV and its local channels from bfmtv.com, M6, W9, Gulli and 6ter from M6+, plus
Equidia, KTO, local stations and a few overseas channels. Every address is found at play time, since most are signed
and short-lived.

**Status: 0.1.0, not released yet.** A live check over the 95 clear channels on 2026-11-20 played every one end to
end, T18 on a second try. A Dailymotion master is refused about once in twenty fetches; the Nightglass relay asks it
again since [PR #44](https://github.com/MarcDownHQ/nightglass/pull/44) (see [TODO.md](TODO.md)). Antenne speaks the
plugin contract 0.5.0 of Nightglass [PR #47](https://github.com/MarcDownHQ/nightglass/pull/47) and
[PR #48](https://github.com/MarcDownHQ/nightglass/pull/48) (Widevine), not merged yet, so it needs a Nightglass
release that includes them. The M6+ channels played at 1080p on 2026-11-26 once two Nightglass decryption bugs were
worked around outside Antenne (see [TODO.md](TODO.md)). No replay and no programme guide yet (see
[ROADMAP.md](ROADMAP.md)).

## Install

Download `antenne.wasm` from the latest
[GitHub release](https://github.com/MarcDownHQ/Antenne/releases) into Nightglass's plugin folder:

- Windows: `%APPDATA%\com.marcdownhq.nightglass\plugins`
- macOS: `~/Library/Application Support/com.marcdownhq.nightglass/plugins`
- Linux: `~/.local/share/com.marcdownhq.nightglass/plugins`

Restart Nightglass, then add a source of type Antenne (one click, from its preset). It has no settings.

### M6, W9, Gulli and 6ter

These four are Widevine and need a free M6+ account:

1. In Nightglass's Settings, turn on protected content (Widevine). Nightglass downloads Google's module; until then
   the four channels are greyed out.
2. Play one of them. Nightglass shows a five-letter code: open [m6.fr/device-pairing](https://www.m6.fr/device-pairing)
   on a phone or a computer, sign in to M6+ and enter it within five minutes.
3. Play the channel again. Antenne is now paired as a TV of that account; it keeps a device id and the account id,
   never a password. Removing the device from the M6+ account brings the code back on the next play.

## How it works

`load-catalog` returns a channel table compiled into the plugin, grouped (National, News, Sport, Thematic,
International, Radio, Regional, Overseas, Local), with the national TNT numbers and logos. `stream-url` resolves the
channel when it is played:

- **France Télévisions**: the k7 descriptor names the live's HLS master, hdfauth signs it.
- **Dailymotion**: the player metadata gives a signed master, played whole (its audio is in separate renditions)
  through the Nightglass relay with a browser handshake. The relay's filter cuts the master down to one encoder,
  since many masters list a primary and a backup side by side on unrelated timelines.
- **BFM**: the live page names a static master.
- **M6+**: Antenne pairs itself with the viewer's account as an M6+ TV, takes the channel's Widevine DASH live and a
  DRMtoday license token, and sends the CDM's license requests to DRMtoday through the `license` export. Nightglass
  decrypts and plays it.
- **The rest**: Kick, the Equidia racing API, EasyBroadcast, `HDR_VISIO` and Ultimedia players, TV Monaco's API,
  and a few fixed addresses.

Details live in [brain/](brain/BRAIN.md). Everything Antenne knows about these services comes from observing them.

## Development

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo clippy --target wasm32-wasip2 --release -- -D warnings
cargo test
scripts/coverage.sh --check                       # 98% of lines
cargo build --release --target wasm32-wasip2       # target/wasm32-wasip2/release/antenne.wasm
cargo run -p antenne-smoke --release -- cnews france-2 bfmtv   # against the live services
```

The toolchain (Rust with the `wasm32-wasip2` target) is pinned in `rust-toolchain.toml` and `.mise.toml`.

## Licence

MIT, see [LICENSE](LICENSE). `wit/provider.wit` comes from Nightglass, MIT as well.
