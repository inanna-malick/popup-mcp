"""What kind of surrounding context actually changes a judgment?

The hypothesis from the last run: context helps when the person's words
can *point at* it, and does nothing (or worse) when it is narration about
the situation. These cases are built to need context the bare packet does
not carry - references to earlier turns, to named things, to options
already declined.
"""
import json, os, pathlib, subprocess, statistics

MODEL = os.environ.get("JEV_MODEL", "jev-latest")
def _key():
    k = os.environ.get("TYPESAFE_API_KEY")
    if k: return k.strip()
    raw = pathlib.Path.home().joinpath(".config/typesafe/jev-key.env").read_text().strip()
    f = raw.splitlines()[0]
    return (f.split("=",1)[1] if "=" in f else f).strip().strip('"')

def ask(state, questions):
    body = json.dumps({"state": state, "model": MODEL, "questions": questions})
    r = subprocess.run(["curl","-sS","https://api.typesafe.ai/v1/systemone",
        "-H", f"Authorization: Bearer {_key()}", "-H","Content-Type: application/json",
        "--data-binary","@-"], input=body, capture_output=True, text=True)
    out = json.loads(r.stdout)
    if "answers" not in out: raise RuntimeError(json.dumps(out)[:300])
    return out["answers"]

SAY = "So the payment tests. How do you want to play it?"
LINES = [
 ("l0", "Patch the fixtures by hand again", "They want the quick fix applied directly to the fixture files, as before."),
 ("l1", "Regenerate them from the sandbox", "They want the fixtures rebuilt from the live sandbox so the drift goes away."),
 ("l2", "Leave the tests and fix the schema drift at source", "They want the underlying schema mismatch addressed rather than the tests."),
 ("l3", "Skip it for now, it's not urgent", "They want to defer the whole thing."),
]

# --- the enrichments ---------------------------------------------------
HISTORY_SUMMARY = [
 {"I said": "What's biting you today?", "they answered": "the payment tests again"},
 {"I said": "Same four as last week?", "they answered": "yeah, plus one in refunds"},
 {"I said": "And you tried the sandbox route on the auth tests?", "they answered": "that worked well actually"},
]
HISTORY_ORDERED = [
 {"position": 1, "I said": "What's biting you today?", "they said": "the payment tests again"},
 {"position": 2, "I said": "Same four as last week?", "they said": "yeah, plus one in refunds"},
 {"position": 3, "I said": "And you tried the sandbox route on the auth tests?",
  "they said": "that worked well actually"},
]
THINGS = [
 {"name": "the sandbox route", "is": "regenerating fixtures from the live sandbox, which they used on the auth tests and liked"},
 {"name": "patching by hand", "is": "editing the fixture files directly, which they have done twice before and complained about"},
 {"name": "the schema drift", "is": "the Stripe API version bump that made the fixtures wrong in the first place"},
 {"name": "the refunds test", "is": "the fifth failing test, new this week"},
]
DECLINED = [
 {"at": "two turns ago", "they turned down": "Let's just pin the old Stripe version"},
 {"at": "last week", "they turned down": "Patch the fixtures by hand again"},
]
ABOUT_THEM = ("They dislike repeating manual work and will say so. They prefer a fix that removes "
              "the cause over one that clears the symptom. They are blunt and use shorthand.")
NARRATION = ("I am helping someone with failing payment webhook tests after a Stripe API version "
             "bump. Four tests were already failing and a fifth appeared in refunds. We have "
             "talked about a few approaches over the last couple of weeks.")

def state(typed, **kw):
    s = {"what I just said": SAY,
         "what they answered": typed,
         "the ways this could go, in order": [
            {"position": i+1, "they say": t, "which would mean": m}
            for i, (_, t, m) in enumerate(LINES)],
         "what this conversation is for": "How should I handle the failing payment tests?"}
    if kw.get("narration"): s["what is going on"] = NARRATION
    if kw.get("summary"): s["how we got here"] = HISTORY_SUMMARY
    if kw.get("ordered"): s["what we have already said, in order"] = HISTORY_ORDERED
    if kw.get("things"): s["things we have been calling by name"] = THINGS
    if kw.get("declined"): s["what they have already turned down"] = DECLINED
    if kw.get("person"): s["what I know about them"] = ABOUT_THEM
    return s

def questions():
    crit = {f"line:{k}": {"they are saying": m} for k, _, m in LINES}
    crit["none_fit"] = {"they are saying": "something clear that is none of the lines above"}
    crit["unclear"] = {"they are saying": "too little to tell which line it is, or two equally"}
    q = {"action": {"type": "choice", "instructions": {
        "question": "Which of these did the person just say?",
        "how to read": "`what they answered` is their own words. A reference like \"the second one\", \"that last thing\", \"same as before\" or a name points into the ordered lists in the state; match it by position or by wording.",
        "each option": "is one of the ways I thought this could go, written as they might say it."},
        "criteria": crit}}
    for k, _, m in LINES:
        q[f"holds:line:{k}"] = {"type": "noul", "instructions":
            f"Taken on its own, is this what the person is saying: {m}?"}
    q["how_many"] = {"type": "score", "instructions":
        "How many of the lines is the person asking for? Several remarks about the same line count as one.",
        "criteria": ["none of them","one of them","two of them","three or more of them"]}
    return q

# (typed, expected line, what it needs)
CASES = [
 ("do what worked on the auth tests", "l1", "earlier turn"),
 ("same as we did last time with auth", "l1", "earlier turn"),
 ("not the hand thing again", "l2|l1", "a previously declined option"),
 ("I'm not doing that by hand a third time", "l2|l1", "history of repetition"),
 ("the sandbox route", "l1", "a named thing"),
 ("fix the actual cause for once", "l2", "their disposition"),
 ("you know how I feel about manual work", "l2|l1", "standing facts"),
 ("what you suggested two turns ago", "none_fit|unclear", "an earlier turn that named a declined option"),
 ("include the refunds one too", "none_fit|unclear", "a named thing not in any line"),
 ("just the quick one, I'm out of time", "l0|l3", "no context needed: control"),
 ("leave it", "l3", "control"),
 ("regenerate from sandbox", "l1", "control, near-verbatim"),
]

CONF, MASS, MARGIN, HOLD = 0.4, 0.4, 0.10, 0.7
def decide(a):
    keys = [f"line:{k}" for k, _, _ in LINES]
    r = a["action"]; p, ch = r["probabilities"], r["choice"]
    exits = p.get("none_fit",0) + p.get("unclear",0)
    rival = max([v for k,v in p.items() if k != ch and k in keys] + [0])
    if (r["confidence"] >= CONF and p.get(ch,0) >= MASS
            and p[ch] - max(rival, exits if ch in keys else 0) >= MARGIN and ch in keys):
        return ch.replace("line:","")
    if ch in ("none_fit","unclear"): return ch
    strong = [k for k in keys if a[f"holds:{k}"]["noul"] >= HOLD]
    if len(strong) == 1: return strong[0].replace("line:","")
    if len(strong) > 1: return "several"
    return "unsure"

VARIANTS = [
 ("E0 bare packet", {}),
 ("E1 + narration", {"narration": True}),
 ("E2 + turns, summarised", {"summary": True}),
 ("E3 + turns, ordered + positions", {"ordered": True}),
 ("E4 + things called by name", {"things": True}),
 ("E5 + what they already declined", {"declined": True}),
 ("E6 + what I know about them", {"person": True}),
 ("E7 everything referenceable", {"ordered": True, "things": True, "declined": True}),
 ("E8 everything at all", {"narration": True, "ordered": True, "things": True,
                           "declined": True, "person": True}),
]

CACHE = pathlib.Path("cache_ctxprobe.json")
cache = json.loads(CACHE.read_text()) if CACHE.exists() else {}
if __name__ == "__main__":
    for tag, kw in VARIANTS:
        ok, confs, misses = 0, [], []
        for typed, want, needs in CASES:
            key = f"{tag}|{typed}"
            if key not in cache:
                cache[key] = ask(state(typed, **kw), questions())
                CACHE.write_text(json.dumps(cache))
            a = cache[key]
            got = decide(a)
            confs.append(a["action"]["confidence"])
            if got in want.split("|"): ok += 1
            else: misses.append(f"{typed[:34]} -> {got} (want {want}, needs {needs})")
        print(f"  {tag:<34} {ok}/{len(CASES)}   mean conf {statistics.mean(confs):.2f}")
        for m in misses:
            print(f"        {m}")
