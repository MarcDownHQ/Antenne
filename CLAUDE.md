# Antenne

A [Nightglass](https://github.com/MarcDownHQ/nightglass) plugin, built as a WebAssembly component
(`wasm32-wasip2`), that adds French live TV channels with no DRM: France Télévisions, Dailymotion lives (CNews,
L'Équipe, the radios), BFM, and local stations. Start with `brain/BRAIN.md` to find how anything works; `ROADMAP.md`
is the agreed backlog, `TODO.md` the small work and open decisions.

## The brain

`brain/` is the committed knowledge base: architecture, features, the services, decisions, glossary. Navigate it
through `brain/BRAIN.md` and the two `index.md` files rather than grepping the tree; the `brain` skill has the full
routine.

- Keep it true to the code. A change that makes a brain doc wrong fixes that doc in the same branch.
- The code wins: if the brain disagrees with reality, correct the brain. The services win over both: they change
  without notice, and `brain/architecture/services.md` records what was observed and when.

## Clean room

What Antenne knows about the services comes from observing them: requests and answers, captured with curl and kept
as trimmed fixtures. Never read, copy or search third-party addon code for these services, and never cite it. A new
fact about a service goes into `brain/architecture/services.md` with the date it was observed.

## Invariants

- **The contract is `wit/provider.wit`**, vendored from Nightglass (its header names the commit). Only `src/wasm.rs`
  touches wit-bindgen; everything else is plain Rust behind the `Http` and `Storage` traits of `src/host.rs`, so it
  builds and is tested natively.
- **Each call runs in a fresh instance.** Anything kept between calls goes through `Storage`, with a time to live no
  longer than what it keeps stays valid (see `brain/features/`). A storage failure never fails playback.
- **Host limits**: `describe` 10 s, other calls 5 min, `filter` 30 s. `filter` runs on every answer of a filtered
  relay, segments included: anything that is not a Dailymotion master passes through untouched and fast.
- **No DRM, ever.** A source that reports DRM (France TV `video.drm`, Dailymotion `protected_delivery`) is an error,
  not something to work around.
- **Tests never reach the network**: they run on `FakeHttp` and the fixtures in `tests/fixtures`. The live check is
  on demand: `cargo run -p antenne-smoke --release -- [CHANNEL...]`.

## Conventions

- `CHANGELOG.md` `[Unreleased]` gets an entry with every user-facing change; it becomes the release notes.
- The Rust toolchain is pinned in `rust-toolchain.toml` and `.mise.toml` (with the `wasm32-wasip2` target); bump both
  together.
- Before finishing: `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`,
  `cargo clippy --target wasm32-wasip2 --release -- -D warnings`, `cargo test`. CI runs the same on the MarcDownHQ
  runner and builds the component.
- Line coverage stays at or above 98% (`scripts/coverage.sh --check`, a CI job). New code comes with tests that keep
  it there; `scripts/coverage.sh` ranks the files with the most uncovered lines.
- Comments only for a constraint the code cannot show. No em-dash anywhere.
- Releases go through the `release` skill.
