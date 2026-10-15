---
target: Popup MCP app
total_score: 30
max_score: 40
na_heuristics:
p0_count: 0
p1_count: 0
target_identity: "file:/Users/inannamalick/dev/popup-mcp/apps/popup/src/ui/PopupForm.tsx"
target_fingerprint: "sha256:6aa786f1d31f3adbd1bcb5478643a97c5159a1fe53df3c30720f4a0ea9eadd84"
target_path: /Users/inannamalick/dev/popup-mcp/apps/popup/src/ui/PopupForm.tsx
timestamp: 2026-10-12T21-17-40Z
slug: apps-popup-src-ui-popupform-tsx
---
Method: dual-agent (A: /root/critique_design · B: /root/critique_evidence)

# Popup design critique — 2026-10-12

Design specificity: authored for this product. Teal selections, continuation rails, compact controls and visible recommendations make conversational alternatives inspectable. Preserve the inspector direction.

| Heuristic | Score | Finding |
|---|---:|---|
| System status | 3 | Clear counts and delivery states; rank ordinal omission fixed after assessment. |
| Real-world match | 3 | Task-specific language; generic form terminology remains. |
| User control | 3 | Clear/cancel/back supported; ranking order requires remove/reselect. |
| Consistency | 3 | Coherent visual system; ranking has a less familiar interaction. |
| Error prevention | 3 | Bounded values and immutable retry; exact numeric entry indirect. |
| Recognition | 3 | Branch context visible; later alternatives can fall below lengthy details. |
| Efficiency | 3 | Keyboard shortcuts supported; numeric and rank editing could be more direct. |
| Minimalist design | 3 | Restrained identity; unequal branch heights leave space. |
| Recovery | 3 | Saved receipt and retry/copy; raw caught errors can still surface. |
| Help | 3 | Contextual guidance present; shortcuts have no visible hint. |
| Total | 30/40 | Good; first baseline, not a measured usability result. |

The score describes Assessment A's reviewed version. The confirmed rank accessibility defect was then fixed; no numerical uplift is claimed without a new independent assessment.

Strengths: branch ownership remains clear; light/dark and narrow/wide preserve identity; delivery ends with a useful receipt and recovery options.

Completed during this pass: stronger control/placeholder contrast, themed caret and selection, visually distinct delivery-error treatment, form busy semantics, ordinal descriptions and live order feedback for rank choices. Browser assertions cover rank updates and light/dark contrast.

Remaining priority issues:
- P2: Long expanded branches push remaining parent alternatives down. Add optional collapse of configured branches with retained selection/summary; preserve simultaneous comparison and DOM order. Suggested workflow: impeccable layout.
- P2: Sliders make known exact numbers indirect. Add editable numeric value chips with existing bounds/step semantics while retaining the sliders. Suggested workflow: impeccable adapt, then polish.

Resolved P1: ranked choices previously exposed selection without their ordinal. Each now describes Priority N of total; a polite status announces the full order. Actual assistive-technology verification remains outstanding.

Cognitive load: three reference-checklist failures in expanded fixtures: Capture has five options (chunking and minimal-choice thresholds); detailed branches interrupt parent-option scanning. Five ordinary checkboxes do not justify forced pagination. Other groups contain at most four choices. Emotional journey: clear opening, rewarding branch reveal, attention valley in long configuration, reassuring receipt and retry.

Personas: Alex encounters indirect numeric entry and remove/reselect ranking. Sam benefits from native controls and now ordinal descriptions; real screen-reader behavior is unverified. Casey faces long narrow forms and lacks a compact configured-branch summary after interruptions.

Minor observations: delivery-error instructions repeat; the submit accelerator is undiscoverable; generic caught errors could use a more consistent plain-language wrapper.

Deterministic scan: successful exit0, JSON [], no findings for apps/popup/src/ui/PopupForm.tsx. Not proof of usability or full accessibility.

Browser evidence: four fixtures at360/720 light and1200 dark, no horizontal overflow in12 initial states. Keyboard ranking and failure→retry worked, with one delivered message. Default light contrast: body14.71:1, primary6.35:1, control/placeholder5.07:1. Dark:13.84:1,7.96:1,7.24:1. Headless Chromium fallback; native browser unavailable, no visible overlay, real screen-reader session, or arbitrary host palette assessment.

Future design questions: how best to collapse configured detail while preserving comparisons; how to expose exact numeric editing without making the inspector heavier. Existing user direction is to refine forward, not revert changes.

Questions skipped: two remaining Priority Issues; continued refinement direction is already established.
