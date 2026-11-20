# France Télévisions lives

France 2 to 5, franceinfo:, Arte, LCP, the 25 France 3 editions, the france.tv channels (Sport, Docs, Séries, La
chaîne des Maternelles), the partners (TV5Monde, France 24, INA, Mieux) and two La 1ère stations. Each entry
carries its broadcast id (`si_id`, a UUID), compiled in: the ids were stable across calls.

## Resolving

`francetv::resolve`, at play time:

1. A signed master kept from a play in the last 300 s (`ftv:live:<si_id>`) is played again without asking.
2. The viewer's country: kept for 6 h (`ftv:country`), else asked of France TV's geolocation; France when that
   fails.
3. The k7 descriptor, with a browser UA, `os=ios` for HLS. An error object (`message`) is told as is, whatever the
   status: that is how k7 refuses a viewer outside the rights area (HTTP 422, code 2009). `video.drm: true` is
   refused; a format other than `hls` is refused.
4. The signature: the token endpoint is `video.token` when it is a string, `video.token.akamai` when it is an
   object, `https://hdfauth.ftven.fr/esi/TA` when there is neither. The answer is JSON `{"url"}` or the bare
   address.
5. The signed master is kept for 300 s and played directly: no relay, no headers.

## How long a signature lasts

The signed master carries two tokens: one in its path, valid 6 hours, and the `hdnea` query, valid 10 minutes. The
master itself is refused once the query has expired, but the variants and segments it names keep playing on the path
token. So a playback lasts hours, while a signed master is only reused for 5 minutes, half its 10.

## Sources

- [src/francetv.rs](../../src/francetv.rs)
- [tests/fixtures/ftv-k7-france-2.json](../../tests/fixtures/ftv-k7-france-2.json)
- [tests/fixtures/ftv-k7-geo-refused.json](../../tests/fixtures/ftv-k7-geo-refused.json)
- [tests/fixtures/ftv-hdfauth.json](../../tests/fixtures/ftv-hdfauth.json)
