# Glossary

**Master**: an HLS playlist listing variants (`#EXT-X-STREAM-INF`) and, on some streams, audio or subtitle
renditions (`#EXT-X-MEDIA`). What a player opens first.

**Variant**: one quality of a stream, a media playlist named by the master. On many Dailymotion lives a variant is
video only.

**Rendition**: an alternative track a master names with `#EXT-X-MEDIA`, grouped by `GROUP-ID`; a variant's `AUDIO`
attribute names the audio group it plays with.

**Ladder**: the set of variants a master lists, from the smallest to the tallest. A cut-down ladder stops at 480p
where the full one reaches 1080p.

**Encoder**: on Dailymotion, the source a set of variants and renditions comes from. A master can list a primary and
a backup encoder; the backup's URIs carry another video id and its audio groups end in `-backup`.

**Relay**: Nightglass fetching a stream on the player's behalf and serving it on 127.0.0.1, with the headers and the
browser handshake the playback asks for.

**Filter**: the `filter` export; with `relay.filter`, every answer of the relayed stream passes through it.

**Browser handshake**: a request sent with a real browser's TLS and HTTP/2 fingerprint (`browser` in the WIT), not
just its User-Agent.

**Source**: in `catalog.rs`, how a channel's live is found: a Dailymotion video, a France TV broadcast id, a BFM
page, an API, a fixed address.

**si_id**: France Télévisions' broadcast id, a UUID, the key to k7.

**k7**: France TV's video descriptor service (`k7.ftven.fr`), which names a live's untokenised manifest.

**hdfauth**: France TV's token service (`hdfauth.ftven.fr`), which signs a manifest address.

**Embedder**: the page a Dailymotion video claims to be embedded in (`embedder=`); owners that restrict embedding
only serve pages of their own site.

**TNT**: French digital terrestrial television; its channel numbers are set by ARCOM, the regulator.
