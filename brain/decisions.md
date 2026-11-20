# Decisions

Each entry is a choice a reader might want to undo, with the reason not to.

## The name

Antenne is French for a TV aerial, and "à l'antenne" means on air: a source of French live channels. One word,
pronounceable in both languages, and the plugin id `antenne` fits the host's lowercase rule.

## Clean room: what Antenne knows comes from the services

The protocol knowledge comes from observing the services: we wrote a factual spec of their requests and answers by
probing them, and implemented from that spec and from fresh probes. The implementation never looked at third-party
addon code for these services. New facts are added the same way, with a date, in
[architecture/services.md](architecture/services.md), and the trimmed answers become fixtures.

## Plain Rust behind traits, wit-bindgen in one module

The resolvers take `&impl Http` and `&impl Storage`, which mirror the WIT imports, so the whole plugin but its glue
builds and is tested natively with fakes, recorded answers and native coverage. `src/wasm.rs` only converts records;
keeping logic out of it is what makes the 98% floor honest, since that file is not measured.

## No settings, one preset

Every channel is public and free; there is nothing to log in to. A preset lets the user add the source in one
click. The catalog is compiled in rather than fetched: it costs no request, cannot fail, and the services have no
list that covers them all anyway.

## Dailymotion: the browser handshake, not HTTP/1.1 and cookies

The survey recommended fetching the master over HTTP/1.1 with the metadata cookies. A plugin cannot pick the HTTP
version, and re-measuring showed the refusals come from the client's fingerprint: with the host's Chrome handshake
the master answered every time over HTTP/2, while the cookies and a `Referer` made refusals likelier. So the
metadata and the master go with `browser: chrome`, the master with no header at all, and the relay presents the
same handshake.

## Dailymotion: the whole master, through a filtering relay

A single variant plays silent on channels whose audio is in renditions, so the player gets the whole master. Masters
that list two encoders on unrelated timelines play out of sync or silent when a player mixes them, so the relay
filters the master down to one encoder. The cost: the relay passes every segment through `filter` in a fresh
instance; `filter` returns anything that is not a Dailymotion master untouched. The other option, one variant plus
its audio as a `track`, avoids the relay but loses adaptive quality.

## France TV: no relay, a signature kept 5 minutes

The signed master plays directly, with no header, so there is no relay. Its `hdnea` query lasts 10 minutes and the
master is refused after it, while the path token keeps variants and segments playing for 6 hours: keeping the
signature 5 minutes saves two requests when zapping back without ever handing out a dead master.

## Fixed addresses where the survey found static ones

KTO, Gong, viàMaTélé, BIP TV, Tébéo, TéléPaese, Figaro TV and Antenne Réunion serve unsigned, unchanging masters,
found by reading each page. Scraping the pages at play time would trade a rare address change for page layouts
that change more often. Rediscovering them is on the roadmap.

## TNT numbers only where confirmed

The survey's numbers were from memory. Only the national TNT channels get one, confirmed against ARCOM's current
numbering; nothing else is numbered rather than guessed.

## Sources

- [src/dailymotion.rs](../src/dailymotion.rs)
- [src/francetv.rs](../src/francetv.rs)
- [src/catalog.rs](../src/catalog.rs)
- [src/wasm.rs](../src/wasm.rs)
