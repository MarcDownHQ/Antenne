---
name: feature-treatment
description: Ship a finished Antenne feature branch the full way - rebase onto master, run a max-effort multi-agent code review, fix everything confirmed, update the brain/ knowledge base, pass the gate (wasm included) and CI, merge keeping the layered commits, and cut a release when a milestone closes. Use when the user says "feature treatment", "treat this branch", "review and merge this feature", or names a worktree/branch to ship.
---

# Feature treatment

The pipeline a finished feature branch goes through before it lands: **rebase -> max review -> fix -> update brain
-> gate -> merge -> release -> clean up**. Nothing merges without an adversarial review and a green gate.

The branch comes from the argument (a branch or worktree name). If none is given, find it: `wt list` and
`git branch` - it is the non-`master` one; worktrees live in `~/repos/Antenne.worktrees/<branch>`. Confirm which
one if ambiguous. Branch names follow `feature/...`, `fix/...`; every commit subject starts with `[<branch>]`.

## 1. Rebase onto master

```bash
cd ~/repos/Antenne.worktrees/<branch>     # or the primary checkout if the branch is there
git fetch origin && git rebase origin/master
```

Resolve conflicts if any. The branch must sit directly on `master` so the review and the merge see only this
feature. Check the size: `git diff --name-only origin/master..HEAD | wc -l` must stay under 149; split along a seam
if it does not.

## 2. Max-effort review (parallel finders)

The diff: `git diff origin/master...HEAD` (exclude `Cargo.lock` from reading; a dependency change is reviewed from
the `Cargo.toml` files).

Spawn **independent finder subagents in parallel** (one Agent tool call with several invocations), each over the
same diff with a different lens. Scale the count to the feature (4-6 is typical):

- **Correctness** - line by line: inverted conditions, off-by-one, `unwrap` or indexing on data from a service,
  retry loops that never end or give up too soon, JSON paths and HTML markers that match the fixtures, error
  messages that say which channel and which service.
- **Host contract** - the invariants in `CLAUDE.md`: logic stays out of `src/wasm.rs`; anything kept between calls
  goes through `Storage` with a lifetime no longer than what it keeps stays valid, and a failed `set` never fails a
  play; `filter` passes everything but a Dailymotion master through untouched and fast (it runs on every segment,
  30 s per call); stream addresses are network schemes, logos http(s); `describe` stays instant.
- **Services and clean room** - does each request match what `brain/architecture/services.md` records (headers,
  browser handshake, parameters)? Is every new service fact dated there? Nothing taken from third-party addon code,
  and nothing citing it. DRM reported by a service is refused, never worked around.
- **Resilience** - a service that changes its page or its JSON gets a clear error, never a panic; a 404 or an
  error page that still looks plausible (BFM) is not trusted; signed addresses are not kept past their expiry.
- **Reuse / simplify** - does new code re-implement `host::encode`, `html::attribute`, `html::between`, the
  `Http::text`/`json` helpers, or a resolver in `sites.rs`? Does a new source belong in an existing module?
- **Tests** - do the tests cover the new branches: the service's error shape, a non-2xx, a missing field, a
  storage hit and a storage refusal? Are new fixtures real answers, trimmed, with signed tokens replaced? Does any
  test reach the network (it must not; see `brain/architecture/testing.md`)?

Each finder returns findings as JSON objects `{file, line, severity, summary, failure_scenario}`, verified (quote
the line), most-severe first. Tell them NOT to fix anything. Then optionally run one **sweep** finder that has the
merged list and hunts only for gaps.

## 3. Fix what's confirmed

Triage the findings. Re-verify a claim before fixing it - finders surface plausible-but-wrong items too. Fix every
confirmed correctness, contract and resilience issue, and the worthwhile quality ones, as new commits on top
(`[<branch>] fix(<area>): ...`), never by rewriting the reviewed layers. For anything real but out of scope, add it
to `TODO.md` or open a GitHub issue (`gh issue create -R MarcDownHQ/Antenne`) rather than dropping it.

## 4. Update the brain

Bring `brain/` in line with what the feature changed - it ships in the SAME merge, not as a follow-up. Apply the
**brain** skill (it has the checklist); the essentials:

- A **Planned** doc the feature built now describes the code, with `## Sources`.
- Channels added or removed -> `brain/features/catalog.md`, the README and changelog counts.
- New service facts -> `brain/architecture/services.md`, dated.
- New non-obvious decision -> `brain/decisions.md` with its why. New term -> `brain/glossary.md`.
- `ROADMAP.md`: tick the items the branch delivered. `TODO.md`: drop the decisions it settled.
- `CHANGELOG.md` `[Unreleased]` has an entry for every user-facing change.

Commit as `[<branch>] docs: ...`, after checking the brain's relative links resolve.

## 5. Gate (must be green before merge)

```bash
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo clippy --locked --target wasm32-wasip2 --release -- -D warnings
cargo test --locked
scripts/coverage.sh                  # at or above the 98% floor, and look for a logic file drifting down the list
cargo build --locked --release --target wasm32-wasip2
```

When the branch touches a resolver, run the live check on the channels it affects and one of each other source:

```bash
cargo run -p antenne-smoke --release -- cnews france-2 bfmtv <the channels touched>
```

Every commit must pass on its own, not just the tip; check from clean exports (a stale `target/` hides a broken
layer):

```bash
w=$(mktemp -d); export CARGO_TARGET_DIR="$w-target"
for c in $(git rev-list --reverse origin/master..HEAD); do
  rm -rf "$w"/* && git archive "$c" | tar -x -m -C "$w"
  (cd "$w" && cargo clippy -q --all-targets --locked -- -D warnings && cargo test -q --locked) \
    >/dev/null 2>&1 && echo "ok   $(git log -1 --format=%s "$c")" || echo "FAIL $(git log -1 --format=%s "$c")"
done
rm -rf "$w" "$w-target"; unset CARGO_TARGET_DIR
```

## 6. Push and let CI run

`origin` is GitHub (`MarcDownHQ/Antenne`); Forgejo pulls a mirror of it.

```bash
git push --force-with-lease origin <branch>   # the rebase rewrote it
gh pr create -R MarcDownHQ/Antenne --head <branch> ... || gh pr edit ...
gh pr checks --watch -R MarcDownHQ/Antenne
```

CI runs on the MarcDownHQ runner: fmt, clippy for the host and for wasm, tests, the component build, and the
coverage floor. It must be green.

## 7. Merge (keep the layers)

Fast-forward `master` onto the branch rather than squashing, so the layers survive:

```bash
cd ~/repos/Antenne
git switch master && git pull --ff-only
git merge --ff-only <branch>
git push origin master                         # GitHub marks the PR merged
```

## 8. Release

When the branch closes a roadmap milestone, run the **release** skill: minor for new channels or behaviour, patch
for fix-only. Otherwise leave the changes in `[Unreleased]`.

## 9. Clean up

```bash
wt remove <branch>                            # worktree and branch
git push origin --delete <branch>
```

## Done when

The feature is on `master`, the brain reflects it, every layer and CI were green, the branch and worktree are gone,
and a release is cut if a milestone closed. Report a one-line summary of what the review fixed.
