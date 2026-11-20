# The other players

Every one of these returns a direct playback: the address, no headers, no relay.

| Source | Channels | How |
| --- | --- | --- |
| BFM | BFM TV, BFM Business, BFM2, 10 local | The live page's `bitmovin-player-container` `data-video-url`. Only a 2xx page counts: an unknown page is a 404 that still names the national live. The master is kept a day, on disk (`bfm:<path>`). |
| Kick | TG+ | The channel info's `playback_url`; "not live" when `livestream` is null. |
| Equidia | Equidia, its 11 racing feeds | The racing API's `primary` (or the older `stream_url_pri`), signed per call. |
| EasyBroadcast | TVR | The event's `stream`; when `token_authentication` is on (a boolean, or the string `"True"`), the token service's answer is appended as a query. |
| `HDR_VISIO` | viàVosges, TV7 Colmar | The player div's `data-url` on the page, then its JSON (`&mode=html`); the DASH `files.auto` with `manifest.mpd` turned into `playlist.m3u8`. |
| Ultimedia | 20 Minutes TV | The player's `dtkplayer-conf` JSON, `video.src.live.src`. |
| TV Monaco | TV Monaco | The media API's `external_stream.url`. |
| Fixed | KTO, Gong, viàMaTélé, BIP TV, Tébéo, TéléPaese, Figaro TV, Antenne Réunion | The address itself, found on each channel's page when the services were surveyed. |

An answer without the expected address is a clear error naming the service ("Kick tgplus is not live", "no
HDR_VISIO player on …", "Equidia equidia-racing9: no live address"), and any non-2xx status comes back as
`http(status)`.

## Sources

- [src/bfm.rs](../../src/bfm.rs)
- [src/sites.rs](../../src/sites.rs)
- [src/html.rs](../../src/html.rs)
- [src/catalog.rs](../../src/catalog.rs)
