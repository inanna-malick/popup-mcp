# jev-lab

Dev-only experiment rig for the decision-tree routing. Needs a TypeSafe key
(`TYPESAFE_API_KEY` or `~/.config/typesafe/jev-key.env`); it is never part of
`cargo test` and never runs in CI.

```
python3 lab.py --parity    # the lab's state matches what the binary compiles
python3 lab.py --run       # ask, cache, score
python3 lab.py --score     # re-score from cache, no network
```

- `trees/*.json` are real popup definitions, so the lab and the product read
  the same files.
- `cases/*.jsonl` carry `expect` and a `rationale` written **before** the run.
  A label that turns out wrong is recorded as a finding, not edited away.
- `cache.json` holds raw answers per (paradigm, tree, case), so decision rules
  and thresholds sweep offline for free. This is the thing that makes the rig
  worth having.
- `report.md` is the current findings.

The parity check exists because the packet used to be assembled in two crates,
and the lab caught the product silently dropping option descriptions from the
state. If `--parity` fails, trust the binary and fix the lab.
