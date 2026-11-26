# M6+ lives

M6, W9, Gulli and 6ter. Their lives exist only as Widevine DASH, so they are the catalog's `protected` channels:
Nightglass greys them out until the viewer turns on protected content (which downloads Google's CDM), then plays
them through its relay, which gets the keys through Antenne's `license` export and hands mpv a clear stream. Each
entry carries its M6+ channel code (`M6`, `W9`, `gulli`, `6T`).

## Signing in: a TV pairing code

Antenne registers as an M6+ smart TV and never sees a password. `m6plus::session`, at the start of every play:

1. The device id: a random UUID kept on disk (`m6:device`), made on the first play.
2. The account: kept on disk (`m6:account`) once paired. Without it, Antenne asks M6+ for a pairing code and fails
   with `action-required`: "Sign in to M6+ with this code", `https://www.m6.fr/device-pairing`, the code, and
   `retry-in` 3 (the TV apps poll every 3 s). The code is kept 300 s, on disk, so a code entered while the app was
   closed still counts. Each later call asks M6+ about that code: still waiting, the same prompt; forgotten (404), a
   new code; paired, the account it names.
3. The box token: a JWT for the device and the account, kept in memory until a minute before it expires (30 minutes).
   A new one comes from `getJwt`; there is no refresh token. Before asking, Antenne checks the account is still among
   the device's paired users: M6+ keeps handing out tokens, and license tokens, after the viewer removed the pairing,
   so this is the only way to notice. A removed pairing drops the account and brings the prompt back.

Every call to the users and auth services is signed: an HMAC-SHA1, keyed by the TV apps' public salt, of the path,
the body's fields sorted and URL-encoded, and the minute (see [services.md](../architecture/services.md)).

## Playing

`m6plus::resolve`:

1. The session, as above.
2. The live: the TV middleware lists the channel's assets; Antenne takes the `delta_dashcenc_h264` one marked `hd`,
   else any of that type. The Yospace (`noyo_*`) ones go through an ad server that does not resolve from France.
3. The license token: DRMtoday's upfront token for the account and the channel (`dashcenc_<code>`), asked with the box
   token. A 401, 403 or 498 drops the box token and asks once more with a new one.
4. The playback: the manifest, no headers, no relay, and `drm` with Widevine, DASH, no certificate, no `pssh` (the
   manifest has both), and a context of `{"channel", "token"}`.

`m6plus::license` posts the CDM's message as is to DRMtoday with the token from the context. DRMtoday answers JSON
whose `license` is the license in base64; Antenne returns the decoded bytes (a non-JSON answer passes as is). A
refused message gets one more try with a fresh token, for a playback that outlived its token. A refusal is told with
the HTTP status and DRMtoday's `x-dt-resp-code`. The service certificate needs nothing special: the CDM's
certificate request goes the same way.

## What plays

On 2026-11-26, through a Nightglass build of PR #48 with Google's Linux CDM 4.10.3112.0 (software, L3), all four
channels played at 1080p with sound, the first picture about 3 to 4.5 s after the app started (including the
pairing check, the license and the relay). Each live uses one key for every track, and DRMtoday gave it to the L3
CDM as usable. Two Nightglass bugs had to be worked around outside Antenne for that run; they are in
[TODO.md](../../TODO.md).

## Sources

- [src/m6plus.rs](../../src/m6plus.rs)
- [src/catalog.rs](../../src/catalog.rs)
- [src/provider.rs](../../src/provider.rs)
