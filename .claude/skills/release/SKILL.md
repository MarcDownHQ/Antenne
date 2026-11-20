---
name: release
description: Cut an Antenne release, the whole shebang - docs sweep (CHANGELOG, README, brain), a live check, then the version bump, gate, commit, annotated tag and push, then watch the Release workflow until the GitHub release carries antenne.wasm. Use when the user says "cut a release", "release", "tag vX.Y.Z", or "the whole release shebang".
---

# Release shebang

Everything between "the code is on master" and "users can download it". Pushing the tag is what builds and
publishes the release, so there is no undo once step 4 runs: a bad tag means a new patch release.

## 0. What a release publishes

A GitHub release on `MarcDownHQ/Antenne` with `antenne.wasm` and `antenne.wasm.sha256`, its notes taken from the
version's `CHANGELOG.md` section. The pipeline is in `brain/architecture/distribution.md`.

## 1. Pre-flight

- On `master`, working tree clean, level with `origin/master` (GitHub). Stash unrelated dirty files and restore
  them afterwards.
- Anything still on a feature branch is not in this release: finish it first (the **feature-treatment** skill) or
  leave it out on purpose.
- Last version: `git describe --tags --abbrev=0` (tags are `vX.Y.Z`; before the first release there is none). Pick
  the next semver: minor for new channels, sources or behaviour, patch for fix-only. A change to the vendored WIT
  that needs a newer Nightglass is a minor bump that says so in the changelog.

## 2. Docs sweep (review against `git diff v<last>..HEAD`)

- **CHANGELOG.md `[Unreleased]`** covers every user-visible change since the last tag: a completeness check against
  `git log v<last>..HEAD --oneline`. That section becomes the release notes verbatim.
- **README.md**: the channel count, the install folders, the status line.
- **brain/**: every doc a change made wrong is fixed (the **brain** skill has the checklist).
- **Dependencies**: `git diff v<last>..HEAD -- '*Cargo.toml'`. A new dependency of the `antenne` crate must build
  for `wasm32-wasip2`.
- Commit the sweep as `[master] docs(release): ...`.

## 3. Live check

The services change without notice, so check them before shipping:

```bash
cargo run -p antenne-smoke --release -- $(python3 -c "import re;s=open('src/catalog.rs').read();print(' '.join(re.findall(r'(?:ch|france_3|bfm_local)\(\s*\"([^\"]+)\"',s)))")
```

A Dailymotion channel that fails with a 403 once and passes on a second run is the known random refusal (`TODO.md`).
Anything else that fails is fixed first, or the channel leaves the catalog with a line in the changelog.

## 4. Cut it

```bash
v=<x.y.z>
sed -i "0,/^version = \".*\"/s//version = \"$v\"/" Cargo.toml     # the workspace version
cargo update -p antenne --offline -q                                # Cargo.lock follows
python3 - "$v" <<'PY'
import sys, datetime
v = sys.argv[1]; p = "CHANGELOG.md"; s = open(p).read()
today = datetime.datetime.now(datetime.UTC).date().isoformat()
open(p, "w").write(s.replace("## [Unreleased]\n", f"## [Unreleased]\n\n## [{v}] - {today}\n", 1))
PY
cargo fmt --all --check && cargo clippy --all-targets --locked -- -D warnings && cargo test --locked \
  && scripts/coverage.sh --check && cargo build --locked --release --target wasm32-wasip2
git commit -am "[master] chore(release): $v"
git tag -a "v$v" -m "Antenne $v"
git push origin master "v$v"
```

Check before committing that the new `## [x.y.z]` section is not empty and that the version in `Cargo.toml` is the
one tagged: the Release workflow refuses a tag that does not match it.

## 5. Watch the Release workflow

```bash
run=$(gh run list -R MarcDownHQ/Antenne --workflow Release --event push --branch v<x.y.z> --limit 1 --json databaseId --jq '.[0].databaseId')
gh run watch "$run" -R MarcDownHQ/Antenne --exit-status
gh release view v<x.y.z> -R MarcDownHQ/Antenne --json assets --jq '.assets[].name'
```

Read a failing job with `gh run view "$run" --log-failed`. A re-run executes the workflow files and the code at the
tag, so it only helps with a transient failure (the runner, the network): `gh run rerun "$run" --failed`. Anything
else is fixed on `master` and shipped as the next patch release; never move or reuse a tag.

## 6. Verify the download

```bash
d=$(mktemp -d) && gh release download v<x.y.z> -R MarcDownHQ/Antenne -D "$d" \
  && (cd "$d" && sha256sum -c antenne.wasm.sha256); rm -rf "${d:?}"
```

## Done when

The tag is pushed, the GitHub release has `antenne.wasm` and its checksum, the checksum verifies, and
`CHANGELOG.md` has a dated section for it. Report the version and the release URL.
