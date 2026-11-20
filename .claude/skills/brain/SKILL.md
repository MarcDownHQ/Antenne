---
name: brain
description: Use and maintain the brain/ knowledge base - the committed docs describing how Antenne works (architecture, the host contract, the services as observed, features, decisions, glossary). Invoke when the user asks how/where something works ("how is a France TV live signed", "why does Dailymotion go through the relay", "which channels are left out"), when onboarding, or when asked to update, audit, or fix the brain after a change. Read brain/BRAIN.md first; navigate via the indexes, not a whole-tree grep.
---

# The brain

`brain/` is the committed knowledge base: how Antenne works, written so you can answer without reading the source.
Use it as the first stop for understanding, and keep it true to the code and to the services.

## Navigating (to answer a question)

1. Open `brain/BRAIN.md` - the entry index. It has a topic table and a **Find by question** table.
2. Drill via the sub-indexes: `brain/architecture/index.md` (the how) and `brain/features/index.md` (the what).
   Don't grep the whole tree; the indexes are the routing layer.
3. Land on the leaf doc. It is self-contained and ends with a `## Sources` list of the real code paths - follow
   those only if the doc isn't enough or you suspect drift.
4. Terms -> `brain/glossary.md`. Rationale / "why" -> `brain/decisions.md`. What a service answers ->
   `brain/architecture/services.md`, which dates every observation.

A doc whose title says **(Planned)** describes the agreed design, not code. Say so when you answer from it.

Structure:

```
brain/BRAIN.md                 root index + find-by-question
brain/glossary.md  decisions.md
brain/architecture/index.md    overview host-contract services testing distribution
brain/features/index.md        catalog dailymotion france-tv players
```

## Maintaining (after a change)

The rule is in `CLAUDE.md` ("The brain"). The code is the source of truth for the docs, and the services are the
source of truth for the code - if a doc disagrees with reality, fix the doc.

- A change that makes a brain doc wrong fixes that doc on the **same branch**.
- A channel added, removed or moved to another source -> the tables in `brain/features/catalog.md` (groups, counts
  by source, the left-out list), the counts in `README.md` and `CHANGELOG.md`.
- A resolver changed -> its feature doc (`dailymotion.md`, `france-tv.md` or `players.md`), and the storage table
  in `brain/architecture/overview.md` when a key, value or lifetime changed.
- A new fact about a service, or a fact that changed -> `brain/architecture/services.md`, with the date it was
  observed and **Corrected** when it contradicts an earlier entry. Never cite third-party addon code (see
  `CLAUDE.md`, "Clean room").
- `wit/provider.wit` re-vendored -> `brain/architecture/host-contract.md` and the commit in the WIT header.
- Release or CI changed -> `brain/architecture/distribution.md`. The gate or the smoke check changed ->
  `brain/architecture/testing.md`.
- New non-obvious decision -> append it to `brain/decisions.md` with its "why". New term -> `brain/glossary.md`.
- Style: dense, skimmable, present tense, normal prose. No em-dashes. Cross-link siblings with relative markdown
  links. Cite source paths instead of restating code. End each doc with `## Sources`.

Before committing, check every relative link in the touched docs resolves:

```bash
python3 - <<'PY'
import glob, os, re
for f in glob.glob("brain/**/*.md", recursive=True):
    for m in re.finditer(r"\]\(([^)#]+)", open(f).read()):
        t = os.path.normpath(os.path.join(os.path.dirname(f), m.group(1)))
        if not m.group(1).startswith("http") and not os.path.exists(t):
            print("broken:", f, m.group(1))
PY
```

## Auditing (drift check on request)

When asked to verify the brain matches reality:

1. Scope it to the relevant leaf doc(s) - don't re-audit everything.
2. For each, read the `## Sources` paths and confirm the doc's claims still hold (sources, storage keys and
   lifetimes, retry counts, the channel tables). Quick cross-checks: `grep -rn 'const ' src` and the tests in
   `src/catalog.rs`.
3. For `services.md`, probe the service again (curl, or `cargo run -p antenne-smoke --release -- <channel>`) and
   date what you saw.
4. Where they diverge, the code (or the service) wins: correct the doc. Note what you changed.
5. If a feature shipped or was removed, update the indexes (`BRAIN.md`, the two `index.md`) too.

For a broad audit, fan out read-only agents (one per architecture or feature doc) that each diff their doc against
its `## Sources`, then apply the fixes.
