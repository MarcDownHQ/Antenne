# Distribution

Antenne ships as one file, `antenne.wasm`, a WebAssembly component for `wasm32-wasip2` built with
`opt-level = "s"`, LTO and stripped symbols.

## CI

`.github/workflows/ci.yml` runs on the MarcDownHQ self-hosted runner (`[self-hosted, zemmourz0zz]`) on every pull
request and every push to `master`. `jdx/mise-action` installs what `.mise.toml` pins: Rust with clippy, rustfmt,
llvm-tools and the `wasm32-wasip2` target, cargo-llvm-cov and ShellCheck. Two jobs: `check` (the gate in
[testing.md](testing.md), ending with the component build) and `coverage`.

## Releases

`.github/workflows/release.yml` runs on a `v*` tag (or by hand with an existing tag). It checks the tag matches the
`version` in `Cargo.toml`, runs the tests, builds the component, writes `antenne.wasm.sha256`, takes the release
notes from the `CHANGELOG.md` section of that version, and publishes a GitHub release with both files. The `release`
skill drives the whole thing.

## Install

The user drops `antenne.wasm` in Nightglass's `plugins` folder and restarts it; see the README.

## Sources

- [.github/workflows/ci.yml](../../.github/workflows/ci.yml)
- [.github/workflows/release.yml](../../.github/workflows/release.yml)
- [Cargo.toml](../../Cargo.toml)
- [.mise.toml](../../.mise.toml)
- [rust-toolchain.toml](../../rust-toolchain.toml)
