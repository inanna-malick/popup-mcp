"""W3: which context elements earn their tokens. Leave-one-out from the
full set, which additive testing cannot do."""
import json, pathlib, sys
import lab
from lab import cases, tree, pages, edges, bucket, TREES, build_state, action_questions, ask, _gate

CACHE = pathlib.Path("cache_ctx.json")
cache = json.loads(CACHE.read_text()) if CACHE.exists() else {}
FULL = ("options", "framing", "where", "why", "earlier")
VARIANTS = [("full", FULL)] + [(f"-{drop}", tuple(c for c in FULL if c != drop)) for drop in FULL] + [("none", ())]

def decide(a, page):
    if a["is:undo"]["noul"] >= 0.7:
        return "unclear"
    hm = a["how_many"]
    if hm["score"] >= 1.5 and hm["confidence"] >= 0.4:
        strong = [e["to"] for e in edges(page) if a[f"holds:{e['to']}"]["noul"] >= 0.45]
        if len(strong) > 1:
            return "several"
    ids = [f"open:{e['to']}" for e in edges(page)]
    r = a["action"]
    if _gate(r, ids):
        ch = r["choice"]
        return ch[5:] if ch.startswith("open:") else {"stay": "unclear", "hand_back": "none_fit"}[ch]
    if a["is:narrow"]["noul"] >= 0.6:
        return "narrow"
    strong = [e["to"] for e in edges(page) if a[f"holds:{e['to']}"]["noul"] >= 0.7]
    return strong[0] if len(strong) == 1 else "unclear"

for tag, ctx in VARIANTS:
    right = wrong = stalled = 0
    for tn in TREES:
        t, P = tree(tn), pages(tree(tn))
        for i, c in enumerate(cases(tn)):
            key = f"{tag}|{tn}|{i}"
            page = P[c["page"]]
            if key not in cache:
                cache[key] = ask(build_state(t, page, c, ctx), action_questions(t, page))
                CACHE.write_text(json.dumps(cache))
            b = bucket(decide(cache[key], page), c["expect"], t, page)
            right += b in ("acted_right", "handed_right"); wrong += b == "acted_wrong"; stalled += b == "stalled"
    print(f"  {tag:<10} right {right}/51   acted_wrong {wrong}   stalled {stalled}")
