# The catalog

`load-catalog` returns `catalog::CHANNELS`, compiled in: no network call, no settings. Each entry has an id (the
channel id Nightglass hands back to `stream-url`), a name, a group, an optional TNT number, an optional logo and a
`Source`, which says how to find its live.

## Channels

99 channels in nine groups:

| Group | Count | Channels |
| --- | --- | --- |
| National | 12 | France 2, 3, 4, 5, M6, Arte, LCP / Public Sénat, Public Sénat (its own Dailymotion feed), W9, Gulli, T18, 6ter |
| News | 7 | BFM TV, CNews, franceinfo:, BFM Business, BFM2, B Smart, France 24 |
| Sport | 15 | L'Équipe, France TV Sport, Sport en France, Equidia, Equidia Racing Trot, Mag, Galop, Racing 1 to 8 |
| Thematic | 7 | France TV Docs, France TV Séries, La chaîne des Maternelles, INA, Mieux, KTO, Gong |
| International | 1 | TV5Monde |
| Radio | 5 | France Inter, Europe 1, RTL2, Fun Radio, Sud Radio |
| Regional | 35 | The 25 France 3 editions, 10 BFM local channels |
| Overseas | 3 | Réunion and Nouvelle-Calédonie La 1ère, Antenne Réunion |
| Local | 14 | viàMaTélé, viàVosges, TV7 Colmar, 8 Mont-Blanc, Moselle TV, TéléNantes, BIP TV, Figaro TV Île-de-France, 20 Minutes TV Île-de-France, Tébéo, TéléPaese, TG+, TVR, TV Monaco |

By source: 42 France TV, 14 Dailymotion, 13 BFM, 12 Equidia, 8 fixed addresses, 4 M6+, 2 `HDR_VISIO`, 1 each
Kick, EasyBroadcast, Ultimedia and TV Monaco.

The four M6+ channels are `protected`: their lives are Widevine, which Nightglass plays only once the viewer turns on
protected content, and they need an M6+ account paired by a code (see [m6plus.md](m6plus.md)). Every other channel
is clear.

## Numbers and logos

Only the national TNT channels carry a number: France 2 (2), France 3 (3), France 4 (4), France 5 (5), M6 (6),
Arte (7), LCP (8), W9 (9), Gulli (12), BFM TV (13), CNews (14), franceinfo (16), T18 (18), L'Équipe (21), 6ter (22). Public Sénat shares LCP's slot and gets
none. The numbers come from ARCOM's current numbering (see [services.md](../architecture/services.md)).

Logos, in order of preference: France TV's square (`carre`) images, Dailymotion account avatars, Télérama's
500x500 icons, Kick's profile picture for TG+. The France 3 editions use the France 3 logo. 38 channels have none.

The catalog's `stream` is `https://antenne.invalid/<id>`, a stand-in: the host drops a channel whose stream is not
a network address, and the real one is only known at play time.

## Left out

| Channel | Why |
| --- | --- |
| Culturebox | France TV has no live for it any more |
| CNews Prime | A part-time live, off air when probed ("Live déconnecté" on both ids) |
| 01net TV | Page gone, no live on its Dailymotion account |
| Guadeloupe, Guyane, Martinique, Mayotte, Polynésie, Saint-Pierre-et-Miquelon, Wallis et Futuna La 1ère | k7 refuses them from mainland France (code 2009) |
| Tahiti Nui Télévision | Its Brightcove "on now" endpoint answers 404 |
| RTL, TV7 Bordeaux, LMTV Sarthe, TLC Cholet, TV Vendée | YouTube lives |
| TL7 | Twitch only |
| TVPI, TVT Val de Loire, TV78, viàOccitanie | No live found |
| TF1, TMC, TFX, TF1 Séries Films, LCI, RMC Story, RMC Découverte, RMC Life, Novo19, CStar | Behind a sign-in or DRM, not done yet |

## Sources

- [src/catalog.rs](../../src/catalog.rs)
- [src/provider.rs](../../src/provider.rs)
- [src/m6plus.rs](../../src/m6plus.rs)
