# Dailymotion lives

CNews, Public Sénat, T18, L'Équipe, B Smart, Sport en France, France Inter, Europe 1, RTL2, Fun Radio, Sud Radio,
8 Mont-Blanc, Moselle TV and TéléNantes. Each entry carries the video id and, when the owner restricts embedding,
the `embedder` page (Public Sénat, T18, L'Équipe, Europe 1, Moselle TV).

## Resolving

`dailymotion::resolve`, at play time:

1. Metadata, with the Chrome handshake and `Referer: https://www.dailymotion.com/`. An `error` in it is final and
   told as is ("Dailymotion: Live déconnecté (…)"); `protected_delivery: true` is refused as DRM.
2. The master, with the Chrome handshake and no header at all: no cookie, no `Referer` (both make refusals more
   likely, see [services.md](../architecture/services.md)).
3. A refused master (403 `E005`) or a network error means new metadata and a new try, up to 5 tries. The error of
   the last try is returned when none answers.
4. A master that answers but whose tallest variant is below the target asks again too: the target is the height
   remembered for this video (`dm:ladder:<video>`), 720 lines when there is none. After 3 masters that answered,
   the fullest is taken. The height reached is remembered for a day, or for an hour when it fell short of the
   target, so a channel whose ladder stops at 480p (8 Mont-Blanc, TéléNantes) learns it and stops asking.
5. The chosen master's text goes to storage for 120 s (`dm:master:<video>`), and the playback is the master's
   address, relayed with the Chrome handshake and the filter on.

The playback is the **whole** master, not a variant: on most channels the audio is in `EXT-X-MEDIA` renditions and
the video variants carry none, so a single variant plays silent.

## The filter

The relay passes every answer through `filter`. `hls::filter` acts only on an answer from
`https://cdndirector.dailymotion.com/cdn/live/video/<id>.m3u8` that starts with `#EXTM3U`; everything else,
segments included, comes back unchanged.

1. If the master kept by `stream-url` has a taller ladder than the relayed one, the kept one is used.
2. Variants are grouped by encoder: the Dailymotion video id in their URI path, or, when the URIs carry none,
   whether their audio group ends in `-backup`. One encoder: the master passes as is.
3. Two or more: keep the encoder whose id is the video asked for; else, when some audio groups are `-backup`, the
   encoder with none; else the encoder whose audio renditions are all named `fre`; else the first listed.
4. The other encoders' variants go, and so do the `EXT-X-MEDIA` lines of audio groups no kept variant uses.

Mixing encoders is what makes a live play out of sync or silent: their timelines are unrelated, and a player that
switches variants (same bandwidth on both) or takes audio from the other group jumps between them.

## Known gaps

- The relay fetches the master again after `stream-url` checked it, and about one fetch in twenty is refused at
  random. The relay does not retry and the player gets the 403 (see [TODO.md](../../TODO.md)).
- The live edge of the audio and video playlists can differ by a segment (3 s); players that pick each edge alone
  may start a few seconds silent. Not handled (ROADMAP, Later).

## Sources

- [src/dailymotion.rs](../../src/dailymotion.rs)
- [src/hls.rs](../../src/hls.rs)
- [tests/fixtures/dm-metadata-cnews.json](../../tests/fixtures/dm-metadata-cnews.json)
- [tests/fixtures/dm-master-cnews.m3u8](../../tests/fixtures/dm-master-cnews.m3u8)
