# The services

What each service answers, as observed by probing it from a residential connection in France. The first survey was
written on 2026-11-20 as a factual spec of requests and answers; building Antenne the same day re-checked every
fact the code relies on and corrected a few (marked **Corrected**). A service can change any day: when one breaks,
probe it again and update this page with the date.

"Browser UA" below means a desktop Chrome User-Agent string (`host::BROWSER_UA`).

## Dailymotion

**Metadata**: `GET https://www.dailymotion.com/player/metadata/video/<id>?embedder=<page, URL-encoded, or empty>`
with `Referer: https://www.dailymotion.com/`. JSON, always 200:

- `qualities.auto[0].url`: the HLS master, `https://cdndirector.dailymotion.com/cdn/live/video/<id>.m3u8?sec=<signed>&dmTs=…&dmV1st=…`.
- `protected_delivery`: false for every live here.
- `error` instead of `qualities` when it cannot play, with `title` and `raw_message`: "Live déconnecté" (exists,
  not broadcasting), "Contenu non disponible" (embedding restricted: pass `embedder` with a page on the owner's
  site), "Contenu privé." (wrong embedder).
- Sets `v1st`, `ts` and `dmvk` cookies, which `dmV1st` and `dmTs` echo.

**Master**: a refused fetch is a 403 `text/html` with `X-Error-Code: E005`.

- **Corrected.** The survey measured HTTP/1.1 with the metadata cookies as reliable and HTTP/2 as nearly always
  refused, with curl. With a browser's TLS and HTTP/2 handshake (wreq emulating Chrome 149 or Firefox 151, as the
  Nightglass host does), the master answered 200 on 35 of 35 fresh metadata with Chrome and 12 of 12 with Firefox,
  over HTTP/2, with no extra header. Adding the metadata cookies made it fail (1 of 12); adding a `Referer` made it
  worse (9 of 23). The app's own client without the handshake got 4 of 23 (16 of 23 with a browser UA and a
  `Referer`). curl over HTTP/1.1 needs a browser UA: curl's own is always refused.
- A signed master that answered 200 still answers 403 about once in twenty fetches at random, and the next fetch of
  the same address succeeds (12 refusals in 244 fetches, over 40 sequences of repeated fetches).
- **Seen live.** Some signed addresses list a cut-down ladder (240p to 480p) and keep listing it on every fetch;
  fresh metadata gives a full one (CNews: 6 then 10 variants). How often varies: that afternoon, CNews gave the
  full ladder on 1 fresh metadata in 19, so three tries mostly end on 480p.

**Master shapes**:

- Audio muxed in the variants, no `EXT-X-MEDIA` (T18, L'Équipe, Fun Radio, B Smart, local stations).
- Audio in `EXT-X-MEDIA:TYPE=AUDIO` renditions, groups `group_A1` (64 kb/s, 240p and 380p) and `group_A2`
  (128 kb/s, 480p and up), `NAME` `fre` or `und`, no `LANGUAGE`. The video variants have no audio: one alone plays
  silent.
- Primary and backup encoders side by side: every variant twice with the same attributes, the backup's audio groups
  suffixed `-backup`, its URIs under another video id (CNews `x3b68jn` and `x5idxxp`, RTL2 `x2tzzpj` and `x7rkbk4`,
  Fun Radio `xxtuy6` and `x7rk9ec`). The two encoders run on unrelated timelines (media sequence numbers far apart,
  `PROGRAM-DATE-TIME` seconds to hours apart). Masters have also been reported with two encoders and no suffix,
  told apart only by the audio `NAME` (`und` against `fre`).
- URIs are absolute, on `live*.<region>.cf.dmcdn.net/sec2(<token>)/…/<video id>/…`, often with a `#cell=` fragment.
  Media playlists are fMP4 (`.m4s`), 3 s segments, with `PROGRAM-DATE-TIME`, no key; they and the segments need no
  header or cookie.

**Lives on an account**: `GET https://api.dailymotion.com/user/<name>/videos?fields=id,title,onair&flags=live` lists
an account's lives and backups; `live_onair=1` only those on air (for CNews it returned only the backup).

## France Télévisions

**Country**: `GET https://geoftv-a.akamaihd.net/ws/edgescape.json` (301 to `geo-info.ftven.fr`) gives
`reponse.geo_info.country_code`.

**Descriptor (k7)**: `GET https://k7.ftven.fr/videos/<si_id>?country_code=<cc>&os=ios&capabilities=drm&diffusion_mode=tunnel_first&offline=false`
with a browser UA. `os=ios` gives HLS. Under `video`: `url` (untokenised, 403 alone), `format` (`hls`), `drm`
(false for every live here), `token` (today the string `https://hdfauth.ftven.fr/esi/TA?format=json`; formerly an
object with an `akamai` URL), `workflow` `token-akamai`. A viewer outside the rights area gets HTTP 422 with
`{"code":2009,"message":"Il semblerait que vous soyez dans une zone où nous n'avons pas les droits de diffusion (France hexagonale)",…}`
(six La 1ère stations from mainland France). **Corrected**: Wallis et Futuna La 1ère, which the survey saw playing
from mainland France, was refused the same way on every call later that day.

**Signature**: `GET <token URL>&url=<URL-encoded video.url>`. With `format=json` the answer is `{"url": "…"}`,
without it the bare address. The signed address adds a path segment (base64 of `exp=…~acl=/*~hmac=…`) and a
`?hdnea=exp=…~acl=/*~hmac=…` query.

- **Corrected.** The two tokens differ: the path token expires after 6 hours, the `hdnea` query after 10 minutes
  (600 s). Two minutes past the query's expiry, the master answered 403 (with or without the query), while a
  variant playlist (with or without the query) and its segments still answered 200. A playback keeps going; a
  signed master is only reusable for a few minutes.

**Streams**: HLS, 6 variants up to 1080p, audio in renditions (`fr`, plus `qaa` original version and `qad` audio
description on the main channels), subtitles, TS segments with clear-key AES-128 (keys fetchable with a plain GET).

**Lists**: `GET https://api-mobile.yatta.francetv.fr/generic/directs?platform=apps` lists every national live with
`si_id`, label and images (`logo` and `carre` on `medias.france.tv`; none for the france.tv-only channels), plus
one-off events with neither `channel` nor `partner`. No channel numbers.

## BFM

`GET https://www.bfmtv.com/<path>` (for example `en-direct/`, `economie/en-direct/`, `en-direct/bfm2/`,
`lyon/en-direct/`). The `bitmovin-player-container` div carries `data-video-url`, the master on
`live-cdn-<code>-euw1.<brand>.bct.nextradiotv.com/master.m3u8`. An unknown path is a 404 page that still names the
national master. `en-direct/bfm2` 301s to `en-direct/bfm2/`. **Seen live**: the master now 307s to a per-session
address (`?bpkio_sessionid=…`); 4 variants up to 540p, muxed audio, TS.

## The other players

- **Kick**: `GET https://kick.com/api/v2/channels/<slug>/info` with `Accept: application/json`,
  `Referer: https://player.kick.com/`, browser UA. `playback_url` is an AWS IVS master with a `token` JWT;
  `livestream` is null when off air; an unknown slug is a 404. TG+ is `tgplus`.
- **Equidia**: `GET https://api.equidia.fr/api/public/racing/equidia-mobileapp-ios-1/<feed>` with
  `User-Agent: Equidia/6036 CFNetwork/1220.1 Darwin/20.3.0` and `Referer: https://fr.equidia.app/`. `primary`
  (formerly `stream_url_pri`) and `backup` are signed hexaglobe addresses. Feeds `equidia-live2`,
  `equidia-racingtrot`, `-racingmag`, `-racinggalop`, `-racing1` to `-racing8`: all answered 200.
- **EasyBroadcast**: `GET https://<tenant>.player.easybroadcast.io/api/events/<event>` gives `stream` and
  `token_authentication`. **Corrected**: a JSON boolean (`false` for TVR), where the survey saw the string
  `"False"`. With tokens on, `https://token.easybroadcast.io/all?url=<stream>` returns a query to append (not
  exercised).
- **`HDR_VISIO`**: the page's `HDR_VISIO` div (double or single quoted class) has
  `data-url="/visionneuse/visio_v9_js.php?key=live1&amp;habillage=<n>"`; that path plus `&mode=html` returns JSON
  whose `files.auto` is a DASH manifest on `*.live-kd.com`; `playlist.m3u8` in place of `manifest.mpd` serves HLS.
  viàVosges (`/Direct.html`, `habillage=2`) and TV7 Colmar (home page, `habillage=0`).
- **Ultimedia (Digiteka)**: the player page has `<script id="dtkplayer-conf" type="application/json">`, the live
  at `video.src.live.src`. 20 Minutes TV's player is
  `https://www.ultimedia.com/deliver/generic/iframe/mdtk/01521514/zone/1/showtitle/1/src/qvrlxxf`.
- **TV Monaco**: `GET https://videos.tvmonaco.com/api/media/v7/media/le-direct` with
  `Referer: https://videos.tvmonaco.com/content/le-direct` gives `external_stream.url` (an okast master) and
  `external_stream.epg_url` (an XMLTV file).
- **Fixed addresses**, all answering a master on 2026-11-20: KTO, Gong, viàMaTélé, BIP TV, Tébéo, TéléPaese (a media
  playlist, no master), Figaro TV Île-de-France, Antenne Réunion, TVR's EasyBroadcast stream, 20 Minutes TV.

## M6+

Observed on 2026-11-26 with the owner's free M6+ account. These lives are Widevine only; there is no clear variant.

**Lives**: `GET https://hbbtv.middleware.6play.fr/6play/v2/platforms/smarttv/services/6play/live?channel=<code>`
with `X-Customer-Name: m6web` (no sign-in needed, the answer is the same with a box token). Codes `M6`, `W9`,
`gulli`, `6T`, comma-separated for several. `<code>[0].live.assets[]` has `type`, `video_quality` and
`full_physical_path`: `delta_dashcenc_h264` `hd` is
`https://sr-m6web.live.6cloud.fr/out/v1/6play/6play-<m6|w9|gulli|6ter>/cmaf_cenc00/dash-short-hd.mpd` (302 to an
`edge-*` host), `sd` is `dash-short-hd720.mpd`; 6ter also lists Yospace `noyo_*` assets. `service_display.images`
carries small logos (none for Gulli).

**Manifests**: dynamic DASH, `cenc`, one `default_KID` for every adaptation set (video, three audio languages),
Widevine and PlayReady `ContentProtection` with a `cenc:pssh`. Video 360p to 1080p (`avc1.640028`, 5 Mb/s at 1080p,
25 fps), 2.56 s segments (64 frames, 120 AAC frames).

**TV signing**: every call to `6play-users.6play.fr` and `front-auth.6cloud.fr` below carries `X-Customer-Name:
m6web`, `x-auth-device-id: <device id>`, `X-Auth-Token-Timestamp: <unix seconds>` and `X-Auth-Token`: lowercase hex
HMAC-SHA1 keyed by the TV apps' public salt `da39a3ee5e6b4b0d3255bfef95601890afd80709`, over the path (with `/v2`),
then the body's fields sorted by name as `key=value` URL-encoded and joined by `&` (nothing for a GET), then
`floor(unix seconds / 60)`. Unsigned calls answer 498.

**Pairing** (box id `deviceid-<device id>`, box type `smarttv`):

- Start: `POST /v2/platforms/smarttv/box` with JSON `{"boxType":"smarttv","boxId":…}` gives `{"token":"<5 letters>"}`.
  The viewer enters it at `https://www.m6.fr/device-pairing`.
- Poll: `GET /v2/platforms/smarttv/boxTypes/smarttv/boxIds/<box id>/tokens/<code>/status`: `{"status":"waiting",
  "uid":null}`, then `{"status":"paired","uid":"<account>"}` once entered (and still `paired` minutes later); 404 for
  an unknown or expired code.
- Paired users: `GET /v2/platforms/smarttv/boxTypes/smarttv/boxIds/<box id>/users`, an array of accounts (`uid`,
  profile fields), `[]` once unpaired.
- Box token: `GET https://front-auth.6cloud.fr/v2/platforms/smarttv/getJwt` signed, plus `x-auth-box-id`,
  `x-auth-box-type: smarttv` and `X-Auth-gigya-uid: <account>`: `{"token":"<JWT>"}`, 30 minutes (`exp - iat` 1800).
- Unpair: `DELETE /v2/platforms/smarttv/users/<account>/boxTypes/smarttv/boxIds/<box id>` with the same headers, 204.
  **After unpairing, `getJwt` still answers 200 with the account in the token, and the upfront token below is still
  granted**: only the paired users list shows the pairing is gone.

**License token**: `GET https://drm.6cloud.fr/v1/customers/m6web/platforms/smarttv/services/6play/users/<account>/live/dashcenc_<code>/upfront-token`
with `Authorization: Bearer <box token>` and `X-Customer-Name: m6web` gives `{"token":"<JWT>"}` for all four codes
(the web platform `m6group_web` also answers). It has no `exp`; it carries the policy: SD and audio tracks allow a
software CDM, HD needs hardware security, output protection not enforced. The same token served several license
calls in a row.

**License**: `POST https://lic.drmtoday.com/license-proxy-widevine/cenc/` with `x-dt-auth-token: <license token>`,
`Content-Type: application/octet-stream` and the CDM's message as the body. The answer is JSON:
`{"status":"OK","license":"<base64>","service_version_info":{…},…}`; with `?specConform=true` it is the bare
license. A service certificate request (`08 04`) is answered the same way. A malformed message or a missing token is
a 400 HTML page with `x-dt-resp-code: 10001`. With Google's Linux CDM 4.10.3112.0 (`secLevel` 3,
`PLATFORM_UNVERIFIED` in `x-dt-client-info`) the license made the live's key usable, and it decrypted 1080p.

## Numbers and logos

- **TNT numbers** (ARCOM numbering since 2025, per its deliberation 2025-06, read off the French Wikipedia table
  that cites it): France 2 is 2, France 3 3, France 4 4, France 5 5, M6 6, Arte 7, LCP 8, W9 9, Gulli 12, BFM TV 13,
  CNews 14, franceinfo 16, T18 18, L'Équipe 21, 6ter 22. No service answer carries a number.
- **Télérama icons**: `https://focus.telerama.fr/500x500/0000/00/01/clear-<n>.png`, 500x500 PNGs; the 20 first used
  answered 200. Added on 2026-11-26 and checked by eye: M6 118, W9 119, Gulli 482, 6ter 1403.
- **Dailymotion avatars**: `owner.avatar_720_url`, `s1/s2.dmcdn.net/u/<hash>/720x720`.

## Sources

- [tests/fixtures/](../../tests/fixtures/README.md)
- [smoke/src/main.rs](../../smoke/src/main.rs)
- [src/m6plus.rs](../../src/m6plus.rs)
