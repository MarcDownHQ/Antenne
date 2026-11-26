# TODO

Small, concrete work: fixes, chores, decisions to make before a roadmap item starts. Features go in
[ROADMAP.md](ROADMAP.md). Delete an entry when it is done.

## Open decisions

- [ ] **Refused Dailymotion masters.** A signed master answers 403 (`E005`) on about one fetch in twenty; the
      Nightglass relay asks a 403, 502 or 503 once more since PR #44. Measure the failure rate left once a Nightglass
      release ships it, and require that release in the README.
- [ ] **Overseas viewers.** France TV's country goes to k7, so a viewer in Guadeloupe would get Guadeloupe La 1ère,
      but the catalog leaves it out. Decide between listing the seven stations for everyone (an error from mainland)
      or a setting.

- [ ] **Nightglass decryption bugs that stop the M6+ channels.** Found on 2026-11-26 against PR #48 (`8f941fd`),
      reported to Nightglass, worked around outside Antenne for the live check. Require the Nightglass release that
      fixes them in the README.
      1. The CDM helper hands Widevine samples with two or more subsample entries, and Widevine 4.10 answers
         `kNoKey` to every one of them; one entry or none decrypts. H.264 frames with several slices fail, so no
         M6+ channel plays. Decrypting a `cenc` sample's protected bytes as one run works.
      2. The helper sends a session's `keys` after the `done` of its `updateSession`; the client reads the keys
         when `done` arrives, so a license can fail with "The licence holds no key Nightglass may use (none)".
- [ ] **Play-time prompts.** Nightglass shows an `action-required` from `stream-url` as a 15 s notice and does not
      ask again, so the viewer must play the channel again after entering the code. Decide with Nightglass whether
      the player should poll like the Sources screen does.

## Small work

- [ ] EasyBroadcast with `token_authentication` on appends the token to the master only; the service also wants it
      on variant addresses that lack it. Untested (TVR has no token): needs a filtered relay if a channel turns it
      on.
- [ ] The TG+ logo is a Kick upload URL that changes when the channel changes its picture; read
      `user.profile_pic` at catalog time if it breaks.
- [ ] France 3 editions all show the France 3 logo; France TV has no per-edition logo.
- [ ] `epg-id` is empty everywhere until M2 decides where guide data comes from.
