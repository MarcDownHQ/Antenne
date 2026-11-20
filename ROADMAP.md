# Roadmap

The agreed backlog, in delivery order, with the decisions taken on each item so they are not lost. Vocabulary
follows [brain/glossary.md](brain/glossary.md); the reasons behind the shape are in
[brain/decisions.md](brain/decisions.md). Small fixes and open questions go in [TODO.md](TODO.md).

## M0 - Foundation

- [x] Single Rust crate building a `wasm32-wasip2` component against Nightglass's `wit/provider.wit` (vendored),
      with the logic in plain Rust behind `Http` and `Storage` traits. A workspace only for the live smoke check.
- [x] CI on the MarcDownHQ runner: format, clippy (host and wasm), tests, the component builds, 98% line coverage.
- [x] Release workflow: a `v*` tag attaches `antenne.wasm` and its SHA-256 to a GitHub release.
- [x] Name chosen: Antenne, the French word for a TV aerial and for a broadcaster's airtime.

## M1 - Live channels

- [x] Compiled-in catalog of the channels the probes found playable: 95 in nine groups, national TNT numbers,
      logos. Decided: no settings, one preset; channels whose live is DRM, YouTube-only, geo-refused from mainland
      France or gone are left out and listed in [brain/features/catalog.md](brain/features/catalog.md).
- [x] Dailymotion: the whole master, relayed with a Chrome handshake and filtered to one encoder; fresh metadata on
      a refused master; a cut-down ladder asked again. Decided: no cookies, no HTTP/1.1 (see decisions).
- [x] France Télévisions: k7 then hdfauth, both token shapes, DRM refused, the signed master kept 5 minutes.
- [x] BFM, Kick, Equidia, EasyBroadcast, `HDR_VISIO`, Ultimedia, TV Monaco and fixed addresses.
- [x] Live smoke check with the host's own HTTP clients (`smoke/`).

## M2 - What is on

- [ ] Programme guide as `catalog.programmes` (the contract allows it). The services only expose "now" for
      France TV (`generic/directs`, one call for every national live) and a full XMLTV for TV Monaco. Decide first
      whether "now and next" is worth a catalog call that can fail.
- [ ] France TV one-off event lives from `generic/directs` (items with neither `channel` nor `partner`), listed
      while they are on. Needs the catalog to make one network call, best effort.

## M3 - More channels

- [ ] La 1ère stations k7 refuses from mainland France (Guadeloupe, Guyane, Martinique, Mayotte, Polynésie,
      Saint-Pierre-et-Miquelon, Wallis et Futuna): offer them to viewers whose France TV country allows them.
- [ ] Part-time lives (CNews Prime) listed with an "off air" error rather than left out.
- [ ] Rediscover fixed addresses from their pages (KTO, Gong, viàMaTélé, BIP TV, Tébéo, TéléPaese) when they move.

## Later

- [ ] Replay (`catchup-url`) for France TV, whose lives keep hours of DVR.
- [ ] Line up audio and video at the live edge on Dailymotion by `PROGRAM-DATE-TIME` in `filter`, if players start
      silent for a few seconds.

## Not planned

- DRM channels (TF1+, M6+, RMC BFM Play, myCanal). Antenne never plays protected content.
- YouTube lives (RTL, TV7 Bordeaux, LMTV, TLC, TV Vendée): getting HLS out of YouTube is its own problem.
- Twitch (TL7's event channel).
