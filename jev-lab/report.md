# Jev routing lab, wave 2

51 labelled cases across four trees, every label and rationale written before
the run. Trees vary on purpose: 3 edges at one hop, 5 edges three hops deep,
8 edges flat, and a note-first tree with no options at all.

## Headline

| paradigm | right | acted wrong | stalled |
|---|---|---|---|
| Choice over **edges** plus companions | 48/51 | 3 | 0 |
| Choice over **actions** plus companions | **50/51** | **1** | 0 |

Both are one call. The action pool wins on the outcome that matters most for a
forward-only tool: with edges, a person who writes "ignore that, I mis-clicked"
is routed onto the page implied by the answer they just disowned. The action
pool clears the page instead.

## What generalised from wave 1

Every wave-1 finding held on at least three of four trees:

- an answer must travel with its option's description
- concrete conditions in the person's vocabulary, not a summary of intent
- ordered context so a reference has something to land on
- a small Choice pool with the modal moves as Nouls beside it
- one wide call, never a second call to break a tie
- a multiplicity Score, not per-edge Nouls, for "several"
- prefill and confirm, never skip a page

## Context leave-one-out (actions, all trees)

| state | right | acted wrong | stalled |
|---|---|---|---|
| everything | 50/51 | 1 | 0 |
| minus the ordered options | 43/51 | 4 | 4 |
| minus the assistant's framing | 48/51 | 2 | 1 |
| minus where each branch goes | 49/51 | 0 | 2 |
| minus why the popup opened | 49/51 | 1 | 1 |
| minus earlier pages | 49/51 | 0 | 2 |
| none of it | 41/51 | 2 | 8 |

Every element pays. The ordered option list pays seven cases on its own, far
more than anything else. Dropping the last three trades a wrong action for two
stalls, which is the safer direction but a net loss.

## The defect the sweep found

The multiplicity Score was being read without regard to its own confidence. On
a page where nothing had been answered it returned a rung of 1.6 at confidence
0.00, and two branches opened. Requiring confidence took the set from 3 wrong
to 1. The plateau runs 0.3 to 0.5, and 0.4 ships.

## Thresholds, swept from cache

| knob | plateau | shipped |
|---|---|---|
| multiplicity confidence | 0.3 – 0.5 | 0.4 |
| multiplicity rung | 1.3 – 1.5 | 1.5 |
| per-edge Noul, once multiplicity is established | 0.45 only | 0.45 |
| per-edge Noul as a standalone path | unchanged 0.45 – 0.75 | 0.7 |

The per-edge Nouls saturate on trees whose conditions share a phrasing: on the
control tree every edge scored above 0.8 on an off-tree question. They are a
last resort, not a primary signal.

## Still wrong

One case, on the note-first tree: "nothing this week, and what I did write
before is rubbish anyway" opens the stuck branch when both stuck and doubt
hold. Negation ("not the typing one, the other two") now passes but for the
wrong reason, since all three branch Nouls clear the floor.

## Not yet run

Breadth beyond 8 edges, a width ceiling past 15 questions, model sensitivity,
and per-action false-positive rates beyond the cases already in the set.


---

# Wave 3: the conversation language

44 cases over the same four trees, ported to nodes and lines. The mix is
harder than wave 2 by construction: a case where the person simply picks
a line no longer exists, because a click is the answer and nothing is
judged. Everything left is free text that has to be placed.

| | right | acted wrong | stalled |
|---|---|---|---|
| all four trees | 40/44 | 1 | 3 |

**Free text places far more sharply than it did.** Every clear case now
returns confidence 1.00 with the winning line at 0.95+ and the others at
0.03. That is the language change paying off directly: the criterion is
the person's own line, so there is no second representation to drift from.

**The tempting simplification is harmful.** Now that the Nouls looked
clean on the conversational tree, it was worth testing whether the
multiplicity Score could be dropped and "several" read from the line
Nouls alone. It collapses: 11 wrong actions out of 44, because the
control tree's Nouls saturate and fire "several" on every case including
an empty answer. Noul saturation is tree-dependent and cannot be trusted
without the Score, exactly as wave 2 found with abstract conditions.

**Gate.** Sweeping confidence found a plateau at 0.3-0.4 worth one case
over 0.5, with margin flat across its whole range. Shipped 0.4 and 0.10.

**The three stalls are honest.** Two are genuinely ambiguous inputs where
handing back is defensible, and one is the known negation weakness.


## Wave 3b: which surrounding context pays

Twelve cases built to need context the bare packet does not carry:
references to an earlier turn, to a thing called by name, to an option
already declined, and three controls needing nothing.

| state | right | mean confidence |
|---|---|---|
| bare packet | 7/12 | 0.66 |
| plus narration about the situation | 7/12 | 0.68 |
| **plus the turns already taken, summarised** | **9/12** | 0.60 |
| plus the same turns, numbered with positions | 7/12 | 0.60 |
| plus the things being called by name | 7/12 | 0.61 |
| plus what they already turned down | 5/12 | 0.57 |
| plus standing facts about the person | 7/12 | 0.64 |
| plus all of the referenceable ones | 8/12 | 0.62 |
| plus everything at once | 7/12 | 0.63 |

**The turns already taken are the only enrichment that pays**, and the
packet already carried them. "Do what worked on the auth tests" points at
something said earlier and at nothing else.

**Numbering the turns gave the gain straight back.** Positions help for a
list someone points at by ordinal, which is what the lines are. History is
pointed at by content, and numbering it adds noise.

**Listing what they already declined is actively harmful**, costing two
further cases. Putting the text of rejected options into the state makes
it compete with the lines actually on offer.

**Everything at once is worse than the single best addition.** Padding
costs, which is the standing warning holding.

The three cases nothing fixed all need negation or preference inference
("not the hand thing again", "you know how I feel about manual work"),
which remain the known weaknesses.

## Wave 4: shaping the packet the way TypeSafe documents it

Four things the docs describe that the compiler was not doing, each
applied as a transform over the packet the binary compiles (so the
baseline is exactly what the product sends):

- `primitives/advanced.md` — **Noul `criteria.true`/`.false`** with `what`
  and `examples`. Our eight Nouls carried instructions and nothing else.
- `primitives/advanced.md` — **Choice options as rubrics** (`what`,
  `not_for`, `examples`).
- `primitives/advanced.md` — **taxonomy walking**: nest the subtree *in
  the option*. We had the destination in state, beside the option.
- `primitives/advanced.md` — **Score levels as `summary` + `signals`**.
- `model-jaggedness/jev-1.13.md` #2 — "does not reliably count".
  `how_many` is a counting question. Replaced with a boundary Noul.

44 cases, four trees.

| variant | right | wrong | stalled |
|---|---|---|---|
| baseline | 39/44 | 2 | 3 |
| baseline, repeat | **40/44** | 1 | 3 |
| noul criteria | 40/44 | 1 | 3 |
| noul criteria, repeat | 40/44 | 1 | 3 |
| choice rubric | 40/44 | 1 | 3 |
| taxonomy in the option | 40/44 | 1 | 3 |
| score levels structured | 40/44 | 2 | 2 |
| no counting | 39/44 | 2 | 3 |
| noul criteria + taxonomy | 41/44 | 1 | 2 |
| noul criteria + no counting | 41/44 | 1 | 2 |
| all four | 41/44 | 1 | 2 |

**The result is a null.** The same packet run twice scored 39 and 40, so
the noise floor on this set is ±1, and every single-transform result sits
inside it. The pairs reach 41 by flipping one hard multiplicity case, and
adding the third and fourth transforms flips nothing further, so that is
one case moving, not two gains stacking.

What this does establish: none of the four hurts, and the three residual
misses are the same three every time — two multiplicity answers and one
preference inference. The set is saturated. Resolving a difference this
size needs cases built on the boundaries these transforms sharpen
(undo-versus-correction, adjacent-versus-covered), not more of the same.
