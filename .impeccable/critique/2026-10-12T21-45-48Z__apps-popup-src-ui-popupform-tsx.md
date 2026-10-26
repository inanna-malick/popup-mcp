---
target: Popup expanded branches and direct controls
total_score: 35
max_score: 36
na_heuristics: 10
p0_count: 0
p1_count: 0
target_identity: "file:/Users/inannamalick/dev/popup-mcp/apps/popup/src/ui/PopupForm.tsx"
target_fingerprint: "sha256:3e98b39fbed7992b170a47f61982ee1c177368f5c82ef67f8c10e3f2d1f0f8fa"
target_path: /Users/inannamalick/dev/popup-mcp/apps/popup/src/ui/PopupForm.tsx
timestamp: 2026-10-12T21-45-48Z
slug: apps-popup-src-ui-popupform-tsx
---
Method: dual-agent (A: /root/direct_design_review · B: /root/direct_evidence_review)

# Popup direct-control critique — 2026-10-12

Fresh independent review of the direct-editing and spatial-refinement pass. Design specificity: authored for a conversational inspector. Teal selections, attached branch rails, readable current values and compact controls explain consequences without branding or decorative competition. Active branches remain expanded by explicit user direction; ordered-column whitespace is accepted, not a defect.

| Heuristic | Score | Evidence |
|---|---:|---|
| System status | 4 | Counts, values, ordinals, sending state and response receipt. |
| Real-world match | 4 | Task-specific language, meaningful units and recommendations. |
| User control | 4 | Direct numeric entry, Escape reset, reorder, clear, cancel and back. |
| Consistency | 4 | Native semantics and shared theme/layout system. |
| Error prevention | 4 | Bounds, normalization, invalid-submit focus and immutable retry. |
| Recognition | 4 | Attached expanded branches, visible values and priority order. |
| Efficiency | 4 | Slider/direct entry, ranking moves/keyboard, responsive density. |
| Aesthetic/minimalist design | 4 | Quiet teal hierarchy and compact inspector rhythm. |
| Error recovery | 3 | Main recovery strong; clipboard fallback was hidden on delivery failure. |
| Help/documentation | n/a | Explicit user-directed exclusion. |
| Total | 35/36 | Excellent; independent heuristic judgment, not measured usability. |

Score provenance: Assessment A assigned35/36 before the review fixes. No post-fix numerical uplift claimed. B returned zero detector findings and browser evidence; its separate technical rubric is not combined into this Nielsen score.

Priority findings addressed:
- P2 recovery: copy failure during delivery failure hid manual-copy instructions in collapsed Delivery details. Now separate copy-error state exposes an adjacent visible alert; browser regression verifies recovery.
- P2 consistency from B: numeric ArrowUp/Down used browser increments rather than authored slider step. Now both controls step consistently; decimal-step regression added.

Strengths: simultaneous branches retain ownership and authored order; numeric chips and rank arrows enable direct manipulation; receipts preserve labeled answers and recovery actions. At1040px, investigation expanded height decreased948→881px and workspace654→629px; story897→904px and release877→884px. These are fixture measurements, not user attention metrics.

Cognitive load: low renderer-driven load in sampled forms. The workspace Capture choice has five options including Other, a mild content-level exception to the reference checklist. Larger initial surfaces are addressed through authoring, not automatic folds. Four authored density pairs retain substantive claims and serialized answers while removing repetition; no fresh model-generation improvement claim.

Personas: Alex gains exact numeric editing and direct ranking; Sam gets native inputs, invalid descriptions, focused recovery and ordinal/order announcements; Jordan sees visible recommendations and concrete actions. Actual screen-reader verification remains outstanding. Emotional journey remains an intelligible decision, visible consequences, then a reassuring receipt. Secondary recovery issue was corrected.

Minor observation: invalid numeric drafts announce inline while typing. Observe with assistive technology before changing this feedback timing; it prevents blur-triggered layout movement swallowing the submit click.

Evidence: A inspected8captures and source independently. B ran CLI detect with JSON[] and fresh Chromium tests for direct controls and packing;64existing screenshots contained no overflow. Native production host, Safari/Firefox, real screen readers/touch/IME are not certified by this pass. No overlay claimed.

Next question: whether validation announcements should defer while a number is temporarily incomplete, guided by actual assistive-technology observation rather than speculative redesign.

Questions skipped: zero unresolved confirmed Priority Issues after the two review fixes; no design-direction decision needed.
