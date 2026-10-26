#!/usr/bin/env python3
"""Jev routing lab. Dev-only: needs a TypeSafe key, never runs in CI.

  python3 lab.py --parity            packet matches the shipped compiler
  python3 lab.py --run [paradigm..]  ask, cache, and score
  python3 lab.py --sweep             re-score from cache, no network

A paradigm is (build_questions, decide). Raw answers are cached per
(paradigm, tree, case) so decision rules and thresholds sweep for free.
"""
import argparse, json, os, pathlib, subprocess, sys

HERE = pathlib.Path(__file__).parent
BIN = HERE.parent / "target" / "debug" / "popup"
CACHE = HERE / "cache.json"
ENDPOINT = "https://api.typesafe.ai/v1/systemone"
MODEL = os.environ.get("JEV_MODEL", "jev-latest")

def _key():
    k = os.environ.get("TYPESAFE_API_KEY")
    if k:
        return k.strip()
    raw = pathlib.Path.home().joinpath(".config/typesafe/jev-key.env").read_text().strip()
    first = raw.splitlines()[0]
    return (first.split("=", 1)[1] if "=" in first else first).strip().strip('"')

def ask(state, questions):
    body = json.dumps({"state": state, "model": MODEL, "questions": questions})
    r = subprocess.run(["curl", "-sS", ENDPOINT, "-H", f"Authorization: Bearer {_key()}",
                        "-H", "Content-Type: application/json", "--data-binary", "@-"],
                       input=body, capture_output=True, text=True)
    if r.returncode:
        raise RuntimeError(r.stderr[:300])
    out = json.loads(r.stdout)
    if "answers" not in out:
        raise RuntimeError(json.dumps(out)[:300])
    return out["answers"]

# ---------------------------------------------------------------- the trees
def tree(name):
    return json.loads((HERE / "trees" / f"{name}.json").read_text())

def cases(name):
    return [json.loads(l) for l in (HERE / "cases" / f"{name}.jsonl").read_text().splitlines() if l.strip()]

def pages(t):
    return {p.get("id") or _snake(p["page"]): p for p in t["elements"]}

def _snake(label):
    return "".join(c if c.isalnum() else "_" for c in label.lower()).strip("_")

def edges(page):
    return [e for e in page.get("next", []) if "if" in e]

def questions_of(page):
    """Answerable elements of a page, as (id, label, options|None)."""
    out = []
    for el in page.get("elements", []):
        kind = next((k for k in ("select", "multi", "input", "slider", "check", "rank") if k in el), None)
        if not kind:
            continue
        opts = el.get("options") if kind in ("select", "multi") else None
        opts = [o["value"] if isinstance(o, dict) else o for o in opts] if opts else None
        out.append((el.get("id") or _snake(el[kind]), el[kind], opts))
    return out

def describe(page, aid, value):
    """An answer travels with the description that sat beside it on screen."""
    for el in page.get("elements", []):
        if el.get("id") != aid:
            continue
        for o in el.get("options", []) or []:
            if isinstance(o, dict) and o["value"] == value and o.get("description"):
                return f"{value} ({o['description']})"
    return value

def framing(page):
    return [el["markdown"] for el in page.get("elements", []) if "markdown" in el]

# ------------------------------------------------------------ the state
def build_state(t, page, case, ctx=("options", "framing", "where", "why", "earlier")):
    P = pages(t)
    answers = {lbl: describe(page, aid, case["answers"][aid])
               for aid, lbl, _ in questions_of(page) if aid in case["answers"]}
    st = {"current_page": page["page"], "current_answers": answers,
          "current_note": case.get("note")}
    if "options" in ctx:
        ordered = []
        for aid, lbl, opts in questions_of(page):
            if not opts:
                continue
            for i, o in enumerate(opts):
                ordered.append({"position": i + 1, "question": lbl,
                                "option": describe(page, aid, o)})
        if ordered:
            st["the choices this page offered, in order"] = ordered
    if "framing" in ctx and framing(page):
        pts = [ln.strip("- *") .strip() for blk in framing(page)
               for ln in blk.split("\n") if ln.strip()]
        st["what the assistant said above the questions, in order"] = [
            {"position": i + 1, "said": p} for i, p in enumerate(pts)]
    if "where" in ctx:
        st["where each branch goes"] = {
            e["to"]: {"this page then asks": [lbl for _, lbl, _ in questions_of(P[e["to"]])]}
            for e in edges(page) if e["to"] in P}
    if "why" in ctx:
        st["why this popup was opened"] = t["title"]
    if "earlier" in ctx:
        st["earlier_pages"] = case.get("earlier", [])
    return st

# ------------------------------------------------------------- paradigms
def _exits():
    return {"none_fit": {"take this branch when": "the answers and the note are clear, but what they describe is not any of the conditions above"},
            "unclear": {"take this branch when": "the answers and the note do not say enough to tell which condition holds, or they support two of them equally"}}

HOW_MANY = {"type": "score", "criteria": ["none of them", "one of them", "two of them", "three or more of them"]}

def _companions(page):
    q = {f"holds:{e['to']}": {"type": "noul", "instructions":
         f"Does this condition hold on its own for the current page: {e['if']}?"} for e in edges(page)}
    q["how_many"] = {**HOW_MANY, "instructions":
        "How many of the branches listed in `where each branch goes` is the person asking for? Several answers about the same branch still count as one."}
    return q

def edge_questions(t, page):
    c = {e["to"]: {"take this branch when": e["if"]} for e in edges(page)}
    P = pages(t)
    for e in edges(page):
        if e["to"] in P:
            c[e["to"]]["the branch then asks"] = [lbl for _, lbl, _ in questions_of(P[e["to"]])][:3]
    c.update(_exits())
    return {"route": {"type": "choice", "instructions": {
        "question": "Which branch condition do `current_answers` and `current_note` describe?",
        "how to read": "`current_note` is the person's own words and outranks a selection it disagrees with. A reference like 'the second one' or 'that last thing' points into the ordered lists in the state; match it by position or by wording.",
        "each option": "gives the condition for taking that branch and what that branch would go on to ask."},
        "criteria": c}, **_companions(page)}

ACTION_EXTRAS = {
 "confirm": "Has the person already said enough to answer the questions on the branch that is about to open, so it can be shown pre-filled for a one-tap confirmation?",
 "undo": "Is the person saying their filled-in answer was a mistake and should simply be cleared, rather than pointing at a different one instead?",
 "expand": "Is the person asking for something close to one of the branches but not quite any of them, so more choices on this page would serve better than moving on?",
 "narrow": "Has the person narrowed it to two of the branches but not to one?",
}

def action_questions(t, page):
    c = {f"open:{e['to']}": {"do this when": e["if"]} for e in edges(page)}
    c["stay"] = {"do this when": "the person has not said enough to show anything new yet, so stay on this page and let them say more"}
    c["hand_back"] = {"do this when": "what the person is talking about is not any of the branches above, so close and let the assistant answer in chat"}
    return {"action": {"type": "choice", "instructions": {
        "question": "What should this window do next?",
        "how to read": "`current_note` is the person's own words and outranks a selection it disagrees with. A reference like 'the second one' or 'that last thing' points into the ordered lists in the state; match it by position or by wording.",
        "each option": "describes the situation that calls for that action."},
        "criteria": c},
        **{f"is:{k}": {"type": "noul", "instructions": v} for k, v in ACTION_EXTRAS.items()},
        **_companions(page)}

CONF, MASS, MARGIN, MANY, HOLD_MANY, HOLD = 0.5, 0.4, 0.15, 1.5, 0.45, 0.7

def _several(a, page, th=None):
    th = th or {}
    if a["how_many"]["score"] < th.get("many", MANY):
        return None
    strong = [e["to"] for e in edges(page)
              if a[f"holds:{e['to']}"]["noul"] >= th.get("hold_many", HOLD_MANY)]
    return "several" if len(strong) > 1 else None

def _gate(r, keys, th=None):
    th = th or {}
    p, ch = r["probabilities"], r["choice"]
    exits = sum(p.get(k, 0) for k in ("none_fit", "unclear", "stay", "hand_back"))
    rival = max([v for k, v in p.items() if k != ch and k in keys] + [0])
    return (r["confidence"] >= th.get("conf", CONF) and p.get(ch, 0) >= th.get("mass", MASS)
            and p[ch] - max(rival, exits if ch in keys else 0) >= th.get("margin", MARGIN))

def edge_decide(a, t, page, th=None):
    th = th or {}
    ids = [e["to"] for e in edges(page)]
    s = _several(a, page, th)
    if s:
        return s
    r = a["route"]
    if r["choice"] in ("none_fit", "unclear"):
        return r["choice"]
    if _gate(r, ids, th):
        return r["choice"]
    strong = [i for i in ids if a[f"holds:{i}"]["noul"] >= th.get("hold", HOLD)]
    return strong[0] if len(strong) == 1 else "unclear"

def action_decide(a, t, page, th=None):
    th = th or {}
    ids = [f"open:{e['to']}" for e in edges(page)]
    if a["is:undo"]["noul"] >= th.get("undo", 0.7):
        return "unclear"
    s = _several(a, page, th)
    if s:
        return s
    r = a["action"]
    if _gate(r, ids, th):
        ch = r["choice"]
        return ch[5:] if ch.startswith("open:") else {"stay": "unclear", "hand_back": "none_fit"}[ch]
    if a["is:narrow"]["noul"] >= th.get("narrow", 0.6):
        return "narrow"
    strong = [e["to"] for e in edges(page) if a[f"holds:{e['to']}"]["noul"] >= th.get("hold", HOLD)]
    return strong[0] if len(strong) == 1 else "unclear"

PARADIGMS = {
 "edges": (edge_questions, edge_decide),
 "actions": (action_questions, action_decide),
}

# --------------------------------------------------------------- scoring
def bucket(got, want, t, page):
    """acted_right | handed_right | stalled | acted_wrong"""
    ok = got in want.split("|") or (got == "narrow" and "|" in want)
    soft = {"unclear", "none_fit"}
    if got in soft:
        return "handed_right" if ok else ("stalled" if want not in soft else "acted_wrong")
    return "acted_right" if ok else "acted_wrong"

TREES = ["triage", "intake", "wide", "chat"]

def run(names, only=None, ctx=None):
    cache = json.loads(CACHE.read_text()) if CACHE.exists() else {}
    for pname in names:
        build, _ = PARADIGMS[pname]
        for tn in (only or TREES):
            t, P = tree(tn), pages(tree(tn))
            for i, case in enumerate(cases(tn)):
                key = f"{pname}|{tn}|{i}" + (f"|{'+'.join(ctx)}" if ctx else "")
                if key in cache:
                    continue
                page = P[case["page"]]
                st = build_state(t, page, case, ctx) if ctx else build_state(t, page, case)
                cache[key] = ask(st, build(t, page))
                CACHE.write_text(json.dumps(cache))
    print(f"cache holds {len(cache)} responses")

def score(names, only=None, ctx=None, th=None, quiet=False):
    cache = json.loads(CACHE.read_text()) if CACHE.exists() else {}
    grand = {}
    for pname in names:
        _, decide = PARADIGMS[pname]
        for tn in (only or TREES):
            t, P = tree(tn), pages(tree(tn))
            tally = {"acted_right": 0, "handed_right": 0, "stalled": 0, "acted_wrong": 0}
            misses = []
            for i, case in enumerate(cases(tn)):
                key = f"{pname}|{tn}|{i}" + (f"|{'+'.join(ctx)}" if ctx else "")
                if key not in cache:
                    continue
                page = P[case["page"]]
                got = decide(cache[key], t, page, th)
                b = bucket(got, case["expect"], t, page)
                tally[b] += 1
                if b in ("stalled", "acted_wrong"):
                    misses.append((b, case.get("note") or case["answers"], got, case["expect"]))
            n = sum(tally.values())
            if not n:
                continue
            grand.setdefault(pname, {"acted_right": 0, "handed_right": 0, "stalled": 0, "acted_wrong": 0})
            for k in tally:
                grand[pname][k] += tally[k]
            if not quiet:
                print(f"  {pname:<8} {tn:<8} right {tally['acted_right']+tally['handed_right']:>2}/{n}"
                      f"  wrong {tally['acted_wrong']}  stalled {tally['stalled']}")
                for b, note, got, want in misses:
                    print(f"      {b:<11} {str(note)[:52]:<54} -> {got:<10} want {want}")
    for pname, tal in grand.items():
        n = sum(tal.values())
        print(f"  TOTAL {pname:<8} right {tal['acted_right']+tal['handed_right']:>2}/{n}"
              f"  acted_wrong {tal['acted_wrong']}  stalled {tal['stalled']}")
    return grand

def parity():
    """The lab's packet must match what the product compiles."""
    bad = 0
    for tn in TREES:
        t = tree(tn)
        page = pages(t)["start"]
        case = cases(tn)[0]
        mine = build_state(t, page, case, ctx=())
        args = [str(BIN), "--file", str(HERE / "trees" / f"{tn}.json"),
                "--compile-packet", "start", "--answers",
                json.dumps({**case["answers"], **({"_note": case["note"]} if case.get("note") else {})})]
        theirs = json.loads(subprocess.run(args, capture_output=True, text=True).stdout)["state"]
        for k in ("current_page", "current_answers", "current_note"):
            if mine.get(k) != theirs.get(k):
                bad += 1
                print(f"  {tn} {k}: lab {mine.get(k)!r} != product {theirs.get(k)!r}")
        # the shipped compiler's question set, for reference
    print(f"  parity: {'ok, state matches on all trees' if not bad else f'{bad} mismatches'}")

if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--parity", action="store_true")
    ap.add_argument("--run", nargs="*")
    ap.add_argument("--score", nargs="*")
    ap.add_argument("--tree")
    ap.add_argument("--ctx")
    a = ap.parse_args()
    ctx = tuple(a.ctx.split("+")) if a.ctx else None
    only = [a.tree] if a.tree else None
    if a.parity:
        parity()
    if a.run is not None:
        run(a.run or list(PARADIGMS), only, ctx)
    if a.score is not None:
        score(a.score or list(PARADIGMS), only, ctx)
