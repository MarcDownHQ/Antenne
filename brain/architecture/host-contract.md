# The host contract

`wit/provider.wit` (package `nightglass:provider@0.2.0`, world `provider`) is vendored from Nightglass; its header
names the source commit. Nightglass is MIT.

## Exports

| Export | Antenne |
| --- | --- |
| `describe` | Id `antenne`, name "Antenne", no fields, `resolves-streams: true`, one preset ("French live TV") with an empty config |
| `load-catalog` | The compiled-in channels; no network call |
| `stream-url` | Resolves the channel's live now (see the features) |
| `catchup-url` | An error: no replay yet |
| `episodes` | An error: live channels only |
| `filter` | Cuts Dailymotion masters to one encoder; everything else unchanged |

## Imports

- `http.send`: http(s) only, made by the app. Any status is a response; only a failed connection is an error.
  Redirects are followed. A request with `browser` is sent with that browser's TLS and HTTP/2 handshake (wreq,
  BoringSSL); without it, by the app's own client (reqwest, HTTP/2 by ALPN). The plugin cannot choose HTTP/1.1.
- `storage.get/set/delete`: text values, optional time to live in seconds, optionally kept on disk. Keys are at
  most 256 bytes, values 1 MiB, 16 MiB in all. Shared by every source of the plugin.
- `log.log`: unused so far; errors go back to the app, which shows and logs them.

## The relay and its filter

A `playback` with a `relay` makes the player open a 127.0.0.1 address: the app fetches the stream itself, with the
playback's headers and the requested handshake, and rewrites HLS playlists so every playlist, segment and key comes
back through it. With `relay.filter`, every 2xx answer (segments included) goes whole through the `filter` export
before the player gets it. A non-2xx answer is passed to the player as is, unfiltered and not retried.

## Limits

- Each call runs in a fresh instance with 256 MB of memory; `describe` has 10 s, `filter` 30 s, the rest 5 min.
- An HTTP answer is at most 128 MB.
- Stream addresses must be network schemes (http, https, rtmp, rtsp, rtp, udp, srt, mms); a channel with any other
  `stream` is dropped from the catalog. Logos must be http(s).

## Sources

- [wit/provider.wit](../../wit/provider.wit)
- [src/wasm.rs](../../src/wasm.rs)
- [src/provider.rs](../../src/provider.rs)
