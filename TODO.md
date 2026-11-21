# TODO

Small, concrete work: fixes, chores, decisions to make before a roadmap item starts. Features go in
[ROADMAP.md](ROADMAP.md). Delete an entry when it is done.

## Open decisions

- [ ] **A refused Dailymotion master fails the play** until Nightglass ships its relay retry: a signed master
      answers 403 (`E005`) on about one fetch in twenty, and the relay now asks a 403 or 5xx once more (commit
      `e0814b6` on `plugin-storage`). Check the failure rate once it lands.
- [ ] **Re-vendor the WIT** once Nightglass merges its `plugin-storage` branch: `wit/provider.wit` comes from commit
      `4b675df` of that branch, not yet on `master`.
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
