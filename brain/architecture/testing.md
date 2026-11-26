# Testing

The gate, run before every commit and in CI:

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --locked --target wasm32-wasip2 --release -- -D warnings
cargo test --locked
cargo build --locked --release --target wasm32-wasip2
```

## Coverage

```bash
scripts/coverage.sh            # summary, then files ranked by uncovered lines
scripts/coverage.sh --check    # fails below 98% of lines (ANTENNE_COVERAGE_FLOOR)
scripts/coverage.sh --html     # browsable report
```

`cargo-llvm-cov` over the `antenne` crate, measured natively. `src/wasm.rs` only builds for wasm and is not counted;
it is kept to conversions with no logic. The smoke crate is not a default member, so it is neither tested nor
counted. CI runs `shellcheck scripts/*.sh` then the check.

## No network in tests

Every resolver takes `&impl Http` and `&impl Storage`. The tests use `testing::FakeHttp`, which answers by exact URL
or by prefix (a key ending in `*`), gives several answers for one key in turn (the last repeating) and records what
was sent, and `testing::FakeStorage`, which records each value's lifetime and can refuse writes.

`tests/fixtures/` holds real answers captured with curl on 2026-11-20, trimmed to what the tests read, with signed
tokens shortened or replaced: Dailymotion metadata and five masters (one per shape), France TV k7 (a live and a geo
refusal), geo and hdfauth, a BFM page fragment, the Kick, Equidia, EasyBroadcast and TV Monaco answers, `HDR_VISIO`
pages and viewer JSON, an Ultimedia player. The M6+ tests use answers written inline in `src/m6plus.rs`, shaped on
those observed on 2026-11-26 (see [services.md](services.md)), since every real one carries an account id or a
token; the request signature is checked against a vector computed independently.

## The live check

```bash
cargo run -p antenne-smoke --release -- cnews france-2 bfmtv lcp
```

`smoke/` implements `Http` with the clients the Nightglass host uses (reqwest, and wreq with Chrome or Firefox
emulation when a request asks for a browser), resolves each channel through `provider::stream_url`, then does what
the relay and the player would: fetch the master with the playback's headers and handshake, pass it through
`provider::filter` when the relay filters, and fetch the first variant and the first audio rendition. It fails a
channel whose master has no variant or no audio. Building it needs cmake (BoringSSL). It cannot check the M6+
channels: they need a pairing and a CDM, so they are checked in Nightglass itself.

## Sources

- [src/testing.rs](../../src/testing.rs)
- [tests/fixtures/README.md](../../tests/fixtures/README.md)
- [smoke/src/main.rs](../../smoke/src/main.rs)
- [scripts/coverage.sh](../../scripts/coverage.sh)
- [.github/workflows/ci.yml](../../.github/workflows/ci.yml)
