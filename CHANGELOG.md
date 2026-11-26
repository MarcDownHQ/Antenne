# Changelog

## [Unreleased]

### Added

- A Nightglass source of 95 French live channels, no settings, added from its preset: France Télévisions
  (France 2 to 5, franceinfo, Arte, LCP, the France 3 editions, the france.tv channels and partners, two La 1ère
  stations), Dailymotion lives (CNews, Public Sénat, T18, L'Équipe, B Smart, Sport en France, France Inter,
  Europe 1, RTL2, Fun Radio, Sud Radio, local stations), BFM TV, BFM Business, BFM2 and ten BFM local channels,
  Equidia and its racing feeds, KTO, Gong, TV Monaco, TG+, TVR and other local channels.
- Addresses are resolved at play time. Dailymotion lives play whole, audio included, through the Nightglass relay,
  cut down to a single encoder.
- M6, W9, Gulli and 6ter (TNT 6, 9, 12, 22) from M6+, as protected channels: Widevine DASH played by Nightglass with
  Google's CDM once protected content is on. Sign-in by the M6+ TV pairing code entered on m6.fr; Antenne keeps a
  device id and the account id, never a password, and asks for a new code when the device is removed from the
  account.
- `antenne.wasm` is attached to each GitHub release.
- Built for the Nightglass plugin contract 0.5.0 (Nightglass PRs #47 and #48); an older Nightglass does not load it.
