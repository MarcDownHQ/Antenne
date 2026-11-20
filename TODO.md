# TODO

Small, concrete work: fixes, chores, decisions to make before a roadmap item starts. Features go in
[ROADMAP.md](ROADMAP.md). Delete an entry when it is done.

## Open decisions

- [ ] **A refused Dailymotion master fails the play.** A signed master answers 403 (`E005`) on about one fetch in
      twenty at random, even with the browser handshake, and an immediate retry of the same address succeeds.
      `stream-url` checks the master, but the Nightglass relay fetches it again and passes a 403 straight to the
      player. Options: (a) the relay retries a refused playlist once (a Nightglass change, the clean fix);
      (b) Antenne returns one video variant plus its audio as a `track`, which never touches the master but loses
      adaptive quality. Leaning (a).
- [ ] **Re-vendor the WIT** once Nightglass merges its `plugin-storage` branch: `wit/provider.wit` comes from commit
      `2fac9af` of that branch, not yet on `master`.
- [ ] **Overseas viewers.** France TV's country goes to k7, so a viewer in Guadeloupe would get Guadeloupe La 1ère,
      but the catalog leaves it out. Decide between listing the seven stations for everyone (an error from mainland)
      or a setting.

## Small work

- [ ] EasyBroadcast with `token_authentication` on appends the token to the master only; the service also wants it
      on variant addresses that lack it. Untested (TVR has no token): needs a filtered relay if a channel turns it
      on.
- [ ] The TG+ logo is a Kick upload URL that changes when the channel changes its picture; read
      `user.profile_pic` at catalog time if it breaks.
- [ ] France 3 editions all show the France 3 logo; France TV has no per-edition logo.
- [ ] `epg-id` is empty everywhere until M2 decides where guide data comes from.
