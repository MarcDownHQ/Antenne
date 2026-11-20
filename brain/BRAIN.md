# Antenne brain

How Antenne works, written so a developer or an agent can answer questions without reading the source. Navigate
through the indexes; every leaf doc ends with the source paths it describes. The code is the source of truth: if a
doc disagrees with it, fix the doc. The services are the source of truth for the code: what they do was observed,
with a date, in [architecture/services.md](architecture/services.md).

Antenne is a Nightglass provider plugin. Nightglass loads `antenne.wasm`, asks it for its channel list, and calls it
again each time a channel is played; Antenne then asks the channel's service for a fresh live address.

## Topics

| Doc | What it covers |
| --- | --- |
| [architecture/index.md](architecture/index.md) | How it is built: modules, the host contract, the services, tests, releases |
| [features/index.md](features/index.md) | What it does: the catalog and each way of finding a live |
| [glossary.md](glossary.md) | The terms the code and the docs use |
| [decisions.md](decisions.md) | Non-obvious choices and why they were made |

## Find by question

| Question | Go to |
| --- | --- |
| Which module does what, and what compiles to wasm? | [architecture/overview.md](architecture/overview.md) |
| What can a plugin do, and what are the host's limits? | [architecture/host-contract.md](architecture/host-contract.md) |
| What does a service answer, and when was that observed? | [architecture/services.md](architecture/services.md) |
| Which channels are in, which are out, and why? | [features/catalog.md](features/catalog.md) |
| Where do the TNT numbers and logos come from? | [features/catalog.md](features/catalog.md) |
| Why does a Dailymotion live go through the relay with a browser handshake? | [features/dailymotion.md](features/dailymotion.md) |
| Why would a Dailymotion live play without sound, or out of sync? | [features/dailymotion.md](features/dailymotion.md) |
| What does `filter` change, and what does it leave alone? | [features/dailymotion.md](features/dailymotion.md) |
| How is a France TV live signed, and for how long does it play? | [features/france-tv.md](features/france-tv.md) |
| What happens with DRM? | [features/france-tv.md](features/france-tv.md), [features/dailymotion.md](features/dailymotion.md) |
| How are BFM, Kick, Equidia and the local players resolved? | [features/players.md](features/players.md) |
| What does Antenne keep in storage, and for how long? | [architecture/overview.md](architecture/overview.md) |
| How is it tested without the network, and how is it checked live? | [architecture/testing.md](architecture/testing.md) |
| How is a release made? | [architecture/distribution.md](architecture/distribution.md) |
| Where did the protocol knowledge come from? | [decisions.md](decisions.md) |
