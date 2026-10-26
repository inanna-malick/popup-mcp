#!/usr/bin/env python3
"""Lab for the conversation language.

The packet comes from the binary, not from here. That is the whole point
of the split: an experiment can never measure a packet the product does
not send. Python owns only the decision rule, which is what gets swept.

  python3 convo_lab.py --run      ask, cache, score
  python3 convo_lab.py --score    re-score from cache, no network
"""
import argparse, json, os, pathlib, subprocess

HERE = pathlib.Path(__file__).parent
BIN = HERE.parent / "target" / "debug" / "popup"
CACHE = HERE / "cache_convo.json"
TREES = ["triage", "intake", "wide", "chat"]
MODEL = os.environ.get("JEV_MODEL", "jev-latest")

def _key():
    k = os.environ.get("TYPESAFE_API_KEY")
    if k:
        return k.strip()
    raw = pathlib.Path.home().joinpath(".config/typesafe/jev-key.env").read_text().strip()
    first = raw.splitlines()[0]
    return (first.split("=", 1)[1] if "=" in first else first).strip().strip('"')

def ask(packet):
    body = json.dumps({"state": packet["state"], "model": MODEL, "questions": packet["questions"]})
    r = subprocess.run(["curl", "-sS", "https://api.typesafe.ai/v1/systemone",
                        "-H", f"Authorization: Bearer {_key()}",
                        "-H", "Content-Type: application/json", "--data-binary", "@-"],
                       input=body, capture_output=True, text=True)
    out = json.loads(r.stdout)
    if "answers" not in out:
        raise RuntimeError(json.dumps(out)[:300])
    return out["answers"]

def compile_packet(tree, node, typed):
    args = [str(BIN), "--file", str(HERE / "convos" / f"{tree}.json"),
            "--compile-packet", node]
    if typed:
        args += ["--typed", typed]
    r = subprocess.run(args, capture_output=True, text=True)
    if r.returncode:
        raise RuntimeError(r.stderr[:400])
    return json.loads(r.stdout)

def cases(tree):
    return [json.loads(l) for l in (HERE / "cases_convo" / f"{tree}.jsonl").read_text().splitlines() if l.strip()]

def lines_of(packet):
    return [k for k in packet["questions"]["action"]["criteria"] if k.startswith("line:")]

# ---- the decision rule, the only thing swept -------------------------------
CONF, MASS, MARGIN = 0.5, 0.4, 0.15
MANY, MANY_CONF, HOLD_MANY, HOLD = 1.5, 0.4, 0.45, 0.7
UNDO, NARROW, EXPAND = 0.7, 0.6, 0.7

def decide(a, keys, th=None):
    th = th or {}
    g = lambda n, d: th.get(n, d)
    act = lambda n: a.get(f"is:{n}", {}).get("noul", 0.0)
    held = lambda k: a.get(f"holds:{k}", {}).get("noul", 0.0)

    if act("undo") >= g("undo", UNDO):
        return "undo"
    hm = a["how_many"]
    if hm["score"] >= g("many", MANY) and hm["confidence"] >= g("many_conf", MANY_CONF):
        strong = [k for k in keys if held(k) >= g("hold_many", HOLD_MANY)]
        if len(strong) > 1:
            return "several"
    r = a["action"]
    p, ch = r["probabilities"], r["choice"]
    exits = p.get("none_fit", 0) + p.get("unclear", 0)
    rival = max([v for k, v in p.items() if k != ch and k in keys] + [0])
    gated = (r["confidence"] >= g("conf", CONF) and p.get(ch, 0) >= g("mass", MASS)
             and p[ch] - max(rival, exits if ch in keys else 0) >= g("margin", MARGIN))
    if gated and ch in keys:
        return ch.replace("line:", "")
    if ch in ("none_fit", "unclear"):
        return ch
    if act("narrow") >= g("narrow", NARROW):
        return "between_two"
    if act("expand") >= g("expand", EXPAND):
        return "adjacent"
    strong = [k for k in keys if held(k) >= g("hold", HOLD)]
    if len(strong) == 1:
        return strong[0].replace("line:", "")
    if len(strong) > 1:
        return "several"
    return "unsure"

SOFT = {"none_fit", "unclear", "unsure", "between_two", "adjacent"}

def bucket(got, want):
    ok = got in want.split("|")
    if got in SOFT:
        return "handed_right" if ok else ("stalled" if want not in SOFT else "acted_wrong")
    return "acted_right" if ok else "acted_wrong"

def run(only=None):
    cache = json.loads(CACHE.read_text()) if CACHE.exists() else {}
    for tree in (only or TREES):
        for i, c in enumerate(cases(tree)):
            key = f"{tree}|{i}"
            if key in cache:
                continue
            p = compile_packet(tree, c["node"], c.get("typed"))
            cache[key] = {"answers": ask(p), "keys": lines_of(p)}
            CACHE.write_text(json.dumps(cache))
    print(f"cache holds {len(cache)} responses")

def score(only=None, th=None, quiet=False):
    cache = json.loads(CACHE.read_text()) if CACHE.exists() else {}
    tot = {"acted_right": 0, "handed_right": 0, "stalled": 0, "acted_wrong": 0}
    for tree in (only or TREES):
        tally = dict.fromkeys(tot, 0)
        misses = []
        for i, c in enumerate(cases(tree)):
            row = cache.get(f"{tree}|{i}")
            if not row:
                continue
            got = decide(row["answers"], row["keys"], th)
            b = bucket(got, c["expect"])
            tally[b] += 1
            tot[b] += 1
            if b in ("stalled", "acted_wrong"):
                misses.append(f'{(c.get("typed") or "(nothing)")[:50]:<52} -> {got:<10} want {c["expect"]}')
        n = sum(tally.values())
        if n and not quiet:
            print(f"  {tree:<8} right {tally['acted_right']+tally['handed_right']:>2}/{n}"
                  f"  wrong {tally['acted_wrong']}  stalled {tally['stalled']}")
            for m in misses:
                print(f"      {m}")
    n = sum(tot.values())
    print(f"  TOTAL    right {tot['acted_right']+tot['handed_right']}/{n}"
          f"  acted_wrong {tot['acted_wrong']}  stalled {tot['stalled']}")
    return tot

if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", action="store_true")
    ap.add_argument("--score", action="store_true")
    ap.add_argument("--tree")
    a = ap.parse_args()
    only = [a.tree] if a.tree else None
    if a.run:
        run(only)
    if a.score or not a.run:
        score(only)
