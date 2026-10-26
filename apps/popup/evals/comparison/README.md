# Conversational authoring comparison — 2026-10-09

This is a historical comparison. The October 10 DSL additions (stable option IDs, recommendations and independent disclosure) were not part of these generations. The runner uses the frozen authoring schemas and the current compiler to retain a compatibility check; it does not claim to evaluate today's live prompt.

The revised guidance teaches partial evaluation of the next useful conversational turns, compatible branches, and substantive editable proposals. This small study found a verbosity regression in the first rewrite. A shorter worked example reduced the initially exposed content in follow-up samples, but **the study does not establish that the final guidance is better than the original prompt**.

## Method

Four fixed requests in `tasks.json` cover CI optimization, dashboard performance, speculative fiction, and an incident with insufficient evidence. Each supplies context the author should use instead of asking for it again. The creative request explicitly seeks exploration rather than a final decision.

Sixteen fresh `gpt-6.1-sol` sessions at medium reasoning effort generated one definition each: two generations per task under each of two authoring bundles. Sessions received no inherited conversation. They were instructed to read only the assigned bundle and their request/context, treat `title` as optional in both variants, and save their first invocation without compiler feedback or repairs. Provider sampling defaults were used; no seed or temperature was controlled.

The frozen inputs are `../baseline-authoring.json` and `../candidate-authoring.json`. This compares complete framing/example packages, not an isolated causal effect of the opening sentence. Schema validation semantics are identical after the title adjustment. Definitions remain in `candidates/`, including any first-attempt failures; none failed in this run.

The runner compiled the candidates, checked the SDK validator, and produced hash-ordered anonymous packets in `blind/`. Four fresh Sol reviewers, one per task, saw the request and four anonymous definitions without their prompts or variant labels. They scored six dimensions from 0–2: context grounding, consequential continuations, composability, meaningful progress, corrections, and attention cost. Higher is better. Raw scores, observations, and candidate-specific answer mappings are in `grades-*.json`; `validation.json` retains the alias mapping.

## Observations

| Bundle | Samples | Compiler + schema valid | Mean initial word proxy | Mean full-tree word proxy |
| --- | ---: | ---: | ---: | ---: |
| Original | 8 | 8/8 | 237 | 297 |
| First rewrite | 8 | 8/8 | 594 | 845 |
| Refined rewrite | 4 | 4/4 | 454 | 826 |

The word proxy counts labels, option descriptions, and input/slider defaults under the compiler's initial visibility or across all branches. It includes generated Other labels but excludes shared notes/button chrome and headings. It is not a measurement of reading time, viewport occupancy, or actual turns saved.

All first-comparison candidates received 2 on the five content dimensions. The rubric therefore had a ceiling effect: it did not distinguish their conversational quality. Attention-cost scores did distinguish them: original mean 2.0, first rewrite mean 0.875. Reviewers repeatedly identified redundant caveats, long editable summaries, and prose repeated in multiple controls. The original prompt produced useful branches on these evidence-rich tasks; this corpus did not reproduce the peer tester's shallow-survey failure.

The final refinement shortened the example itself, opened only its grounded caching recommendation initially, moved further contributions into branches, replaced questions about inspectable repository facts with choices about the next investigation, and included the topic-selector/“Tell me more” failure case. The final bundle is `../final-authoring.json`. Four additional fresh sessions generated one sample per task, followed by a separate reviewer without prompt/history access. These were not a second balanced A/B study. All four were valid and received 2 on content dimensions and 1 on attention cost. CI/performance exposed less text than the first rewrite; creative exploration and incident investigation remained verbose. The full tree remained long.

## Executed checks

`npm run eval:comparison` revalidates the frozen comparison and its final samples against their recorded schemas, then replays four outcomes for every candidate: combined directions, Other, conflicting notes, and cancellation. Eighty result envelopes are recorded in `replays/`. Combined and Other selections use each candidate's actual IDs/options, chosen by the reviewers. The real serializer verifies answer validity, hidden-value omission, and preservation of free-text corrections. Cancellation fixtures contain no answers.

These are deterministic result replays, not independently sampled model continuations. They do not demonstrate that an agent acted correctly after submission. Existing protocol/browser tests separately cover waiting, delivery failure, retry, completed receipts, and terminal immutability.

The new browser scenarios cover simultaneous titleless branches, three levels at 360px width, restored drafts, omitted inactive answers, correction input, host styling, focus, reduced motion, and resize notifications. Light/dark/mobile visual inspection used the final worked example. The app's contract remains one terminal submission with local conditional expansion.

## Limits and next evidence

This is a small, same-model study with coarse subjective scores, no independent model family, no actual chat-host generation, and no user time or task-success measurements. The refinement was informed by these same tasks, so follow-up samples are not held-out validation. Longer branch content can be worthwhile, but word count alone cannot establish that tradeoff. Keep the raw artifacts and the negative result; the next useful evidence is the peer tester's real interaction and short-context prompts that previously elicited generic surveys.
