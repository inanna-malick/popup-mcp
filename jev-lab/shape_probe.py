#!/usr/bin/env python3
"""Does shaping the packet the way TypeSafe's docs recommend actually pay?

Four things the docs describe that this packet was not doing:

  advanced.md   Noul criteria: `true`/`false` objects with `what` and
                `examples`, "when subtle yes/no boundaries must be
                evaluated accurately". Our modal Nouls carry instructions
                and nothing else.
  advanced.md   Choice options as rubrics: `what`, `not_for`, `examples`.
  advanced.md   Taxonomy walking: nest the subtree *in the option*, so the
                branch is judged on where it leads. We put the destination
                in state instead, beside the option rather than in it.
  advanced.md   Score levels as objects with `summary` and `signals`.
  jev-1.13.md   "Doesn't reliably count." `how_many` is a counting
                question, which is the documented weak spot.

Each variant is a transform over the packet the binary compiled, so the
parity guarantee holds: the baseline is exactly what the product sends.

  python3 shape_probe.py --run [--variant NAME]
  python3 shape_probe.py --score
"""
import argparse, copy, json, pathlib
import convo_lab as L

HERE = pathlib.Path(__file__).parent
CACHE = HERE / "cache_shape.json"

def lines(p):
    return [k for k in p["questions"]["action"]["criteria"] if k.startswith("line:")]

def said(p, k):
    return p["questions"]["action"]["criteria"][k]["they are saying"]

# ---- variants --------------------------------------------------------------

def v_noul_criteria(p):
    """Pin the yes/no boundaries the way advanced.md describes."""
    q = p["questions"]
    for k in lines(p):
        q[f"holds:{k}"]["criteria"] = {
            "true": {"what": f"Their words amount to this, however differently worded: {said(p, k)}",
                     "examples": ["a paraphrase of it", "a restatement in their own vocabulary",
                                  "pointing at it by position, like \"the second one\""]},
            "false": {"what": "Their words point at one of the other lines, or say too little to tell.",
                      "examples": ["a different line's wording", "a question about what is going on",
                                   "something none of the lines cover"]}}
    q["is:undo"]["criteria"] = {
        "true": {"what": "They are taking back what they filled in, with nothing put in its place.",
                 "examples": ["ignore that", "scratch that", "I filled that in wrong"]},
        "false": {"what": "They are pointing at a different line. A correction that names a "
                          "replacement is a choice, not an undo.",
                  "examples": ["no, the other one", "actually the second one"]}}
    q["is:narrow"]["criteria"] = {
        "true": {"what": "They have ruled out the rest but are still between two of the lines.",
                 "examples": ["it's one of the first two", "either of those, not the last"]},
        "false": {"what": "They have landed on one line, or have not ruled anything out.",
                  "examples": ["the first one", "I don't know"]}}
    q["is:expand"]["criteria"] = {
        "true": {"what": "What they want sits next to a line but is not it, so more choices here "
                         "would serve them better than moving on.",
                 "examples": ["like that but for the other repo", "sort of, except it's about staging"]},
        "false": {"what": "One of the lines already covers it, or they want something else entirely.",
                  "examples": ["yes, that one", "none of these"]}}
    q["is:confirm"]["criteria"] = {
        "true": {"what": "They already answered what comes after the line they are taking.",
                 "examples": ["the second one, and yes do the refunds too"]},
        "false": {"what": "They answered this turn and nothing beyond it."}}
    return p

def v_choice_rubric(p):
    """Option objects as rubrics: what it covers, what it does not."""
    crit = p["questions"]["action"]["criteria"]
    ks = lines(p)
    texts = {k: said(p, k) for k in ks}
    for k in ks:
        crit[k] = {"what": f"They are saying: {texts[k]}",
                   "not_for": [texts[o] for o in ks if o != k]}
    return p

def v_taxonomy(p):
    """Nest the continuation inside the option, not beside it in state."""
    crit = p["questions"]["action"]["criteria"]
    ways = p["state"]["the ways this could go, in order"]
    for k, way in zip(lines(p), ways):
        onward = way.get("then I ask") or way.get("then")
        if onward is not None:
            crit[k]["and then I would ask"] = onward
        way.pop("then I ask", None)
        way.pop("then", None)
    return p

def v_score_levels(p):
    """Score levels as summary + signals, per advanced.md."""
    p["questions"]["how_many"]["criteria"] = [
        {"summary": "none of them", "signals": ["they are asking something else",
                                                "they are answering a question, not choosing"]},
        {"summary": "one of them", "signals": ["one line, however much they say about it",
                                               "several remarks all about the same line"]},
        {"summary": "two of them", "signals": ["two lines joined by and", "both of those"]},
        {"summary": "three or more of them", "signals": ["all of them", "every one of those"]}]
    return p

def v_no_counting(p):
    """jev-1.13 #2: the model does not count. Ask the boundary instead."""
    p["questions"]["several"] = {"type": "noul", "criteria": {
        "true": {"what": "They are asking for more than one of the lines at once, not just one.",
                 "examples": ["all of these", "the first and the third", "both of those"]},
        "false": {"what": "They are pointing at a single line, however much they say about it.",
                  "examples": ["the second one", "yes, that, and it's urgent"]}},
        "instructions": "Is the person asking for more than one of the lines in `the ways this could go`?"}
    return p

VARIANTS = {"baseline": lambda p: p, "baseline2": lambda p: p, "noul_criteria2": v_noul_criteria, "noul_criteria": v_noul_criteria,
            "choice_rubric": v_choice_rubric, "taxonomy": v_taxonomy,
            "score_levels": v_score_levels, "no_counting": v_no_counting,
            "combo": lambda p: v_no_counting(v_noul_criteria(v_taxonomy(p))),
            "nc_tax": lambda p: v_noul_criteria(v_taxonomy(p)),
            "nc_count": lambda p: v_no_counting(v_noul_criteria(p)),
            "combo_rubric": lambda p: v_choice_rubric(v_no_counting(v_noul_criteria(v_taxonomy(p))))}

# ---- decision rule ---------------------------------------------------------

def decide(a, keys, variant):
    """The shipped rule, except that `no_counting` reads multiplicity from
    the Noul instead of the Score."""
    if "several" not in a:
        return L.decide(a, keys)
    a = dict(a)
    a["how_many"] = {"score": 2.0 if a["several"]["noul"] >= 0.6 else 1.0, "confidence": 1.0}
    return L.decide(a, keys)

# ---- run -------------------------------------------------------------------

def run(names):
    cache = json.loads(CACHE.read_text()) if CACHE.exists() else {}
    for tree in L.TREES:
        for i, c in enumerate(L.cases(tree)):
            base = None
            for v in names:
                key = f"{v}|{tree}|{i}"
                if key in cache:
                    continue
                if base is None:
                    base = L.compile_packet(tree, c["node"], c.get("typed"))
                p = VARIANTS[v](copy.deepcopy(base))
                cache[key] = {"answers": L.ask(p), "keys": lines(p)}
                CACHE.write_text(json.dumps(cache))
    print(f"cache holds {len(cache)} responses")

def score(names):
    cache = json.loads(CACHE.read_text()) if CACHE.exists() else {}
    print(f"  {'variant':<14} {'right':>6} {'wrong':>6} {'stalled':>8}")
    detail = {}
    for v in names:
        tot = dict.fromkeys(["acted_right", "handed_right", "stalled", "acted_wrong"], 0)
        misses = []
        for tree in L.TREES:
            for i, c in enumerate(L.cases(tree)):
                row = cache.get(f"{v}|{tree}|{i}")
                if not row:
                    continue
                got = decide(row["answers"], row["keys"], v)
                b = L.bucket(got, c["expect"])
                tot[b] += 1
                if b in ("stalled", "acted_wrong"):
                    misses.append(f'{tree}/{i} {(c.get("typed") or "")[:44]:<46} -> {got:<11} want {c["expect"]}')
        n = sum(tot.values())
        right = tot["acted_right"] + tot["handed_right"]
        print(f"  {v:<14} {right:>3}/{n:<3} {tot['acted_wrong']:>6} {tot['stalled']:>8}")
        detail[v] = misses
    return detail

if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", action="store_true")
    ap.add_argument("--score", action="store_true")
    ap.add_argument("--misses", action="store_true")
    ap.add_argument("--variant", action="append")
    a = ap.parse_args()
    names = a.variant or list(VARIANTS)
    if a.run:
        run(names)
    d = score(names)
    if a.misses:
        for v, ms in d.items():
            print(f"\n{v}:")
            for m in ms:
                print("   ", m)
