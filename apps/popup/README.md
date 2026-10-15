# Popup MCP App

Agents continue a conversation through interactive messages: grounded analysis, editable proposals and likely follow-ups appear together. Compatible approaches can be explored simultaneously. Submit returns one terminal answer to the conversation.

The visual idiom is a compact inspector for possible next moves: aligned property rows, sliders, checkboxes, tabular values, and persistent branch guides. Wide hosts align labels beside controls; narrow hosts stack them. Selection, recommendation, focus, and disclosure have distinct visual roles. Radio choices can be cleared without losing branch drafts. Motion is restrained, and touch controls retain larger targets.

## Connect

Connect an MCP Apps client to **https://popup-mcp-app.inanna-c38.workers.dev/mcp**. No account or API key is required. Ask the agent to use `popup` to help you make a decision, including follow-up questions for different answers.

The Worker stores no forms or answers. Answers become messages in the host conversation. Open drafts live only in the iframe; a browser-local receipt helps recognize a completed form without storing its answers. A client that cannot accept app messages gets a selectable/copyable response instead. Durable delivery, draft recovery across reloads, and global exactly-once submission are not promised.

## Develop and test

Use Node 24.11+ (or Node 22.18+) and npm.

```sh
cd apps/popup
npm ci
npm run dev
```

This builds the app and starts Wrangler at `http://localhost:8787/mcp`. Run `npm run dev:ui` in another terminal when editing the UI. The frontend is a single HTML resource containing its scripts and styles; the Worker serves it through MCP resources.

```sh
npm run check
npm test
npm exec playwright install chromium
npm run build
npm run test:browser
# With the local Worker still running:
npm run test:worker
POPUP_MCP_URL=http://127.0.0.1:8787/mcp npm run test:browser
npm run deploy:check
npm run deploy
```

`test:browser` uses the official MCP Apps AppBridge in a restrictive iframe sandbox. With `POPUP_MCP_URL`, it additionally fetches the actual tool result and app resource over HTTP before submitting an answer. `test:worker` also accepts `POPUP_MCP_URL` to check a deployment. The test host is a development fixture and is not publicly routed by the Worker.

Adjacent short parameter fields and checkboxes pack into rows when their containing section has room. Adjacent groups of controls can share two columns; compact choices form balanced grids. Multiselects with at least two authored option branches use two columns at a container width of 880px, keeping each option and its entire follow-up subtree together. Other spans the row; shared follow-ups remain outside the grid. Eligibility depends on the definition, never current selections. Exclusive choices retain their existing layout. Text and Markdown outside branches remain full width. Packing uses container widths, so a narrow group inside a wide pane does not inherit a cramped wide-screen layout. Standalone property rows align labels beside controls from 560px; branch-grid parameters keep labels above controls. CSS changes the layout without reordering or remounting controls; resizing preserves drafts, disclosures, and keyboard order. Rows accept unequal branch heights: there is no masonry or dense backfilling.

Short sliders pair from 400px of available section width. Their numeric chips support direct entry: Enter or blur commits a valid value on the configured step, Escape restores the previous answer, and invalid active values block submission with inline feedback. Hidden conditional drafts are retained but do not block submission or enter the result. Explicit off-grid defaults remain untouched until edited. Ranking shows selected priorities in their answer order, with move buttons and Alt+Up/Down; focus follows the moved item and assistive technology receives ordinal descriptions and order updates.

Active branches remain expanded. The author chooses the content and initial scope; the renderer never folds configured answers automatically. Authoring guidance and four paired examples in `evals/density.ts` remove repeated framing while preserving reasoning and answers. This is authored deterministic evaluation, not evidence of improved generation from fresh model samples.

## Visual authoring loop

```sh
npm run visual -- .visual/my-revision
# One authored interaction:
npm run visual -- .visual/story-review story
# Read the deployed tool/resource over MCP instead of loading the local UI build:
POPUP_MCP_URL=https://popup-mcp-app.inanna-c38.workers.dev/mcp npm run visual -- .visual/live
```

Open the output directory's `index.html` to browse the gallery. `evals/visual/calls.mjs` contains four authored `popup` calls: a configurable investigation, an editorial proposal, a release panel, and a grouped investigation workspace. The runner preserves calls, submitted envelopes, screenshots, and a dimensions/overflow manifest. It captures initial and expanded states in 720px light/dark, 1040px light/dark, 1440px, and 360px touch layouts, plus failed-delivery and retried-submission states on desktop. A 720 CSS-pixel capture at device scale 2 approximates the reflow and raster density of a 1440px display at 200% zoom; it does not automate browser zoom. It uses the real UI inside the official AppBridge iframe harness. With `POPUP_MCP_URL`, it also invokes the actual MCP tool and reads its served UI resource. Without it, the harness supplies the opening tool result locally. The temporary preview server is cleaned up on exit; generated files under `.visual/` are ignored by Git.

The Impeccable layout pass reviewed these representative renders with Astra and the implementing agent, then corrected branch parameter alignment and compounded branch spacing in one batch. Branches remain visually connected, and uneven empty space preserves reading order. The bundled Impeccable engine was unavailable, so this pass used source inspection and rendered/browser evidence rather than its mechanical detector. This is qualitative review, not a measured usability study.

Author a plausible next interaction before adjusting styles. Render it, operate the branches, and inspect both the screen and the returned answer. Review the visual output with fresh-context critics who have not been told why the design is good; then compare their observations with your own. Revise coherent typography, state color, geometry, and spacing together. Check transfer to the other calls, dark mode, narrow touch layout, and completion. Run functional browser tests separately; screenshots do not prove keyboard or delivery behavior.

The first recorded loop compared a monochrome baseline with two instrument-style revisions. Fresh Luna reviewers inspected the baseline and first revision; Astra and the implementing agent reviewed the actual renders. Feedback drove consistent teal active states, semantic controls with matching geometry, quieter measurement typography, clearer completion feedback, tighter vertical spacing, and narrower branch gutters. At 720px, the expanded investigation changed from 1024px (baseline) to 1176px (first pass) to 1100px (refinement): a deliberate small density tradeoff for stronger hierarchy. The story call was also corrected after rendering exposed inconsistent object imagery. This is qualitative design review, not a blinded preference study.

The subsequent taste pass concentrates emphasis on decisions: ordinary labels use medium weight, legends and selected choices remain stronger, range rails are 2px, and control outlines use a separate mixed-color token. Shared 3px control corners and 2px numeric-chip corners keep the instrument character precise. Titles use a slightly larger, less tightly tracked treatment. No font assets, animation, or dependencies were added. Astra and the implementing agent reviewed representative light/dark/narrow captures; browser checks verify default-theme text and control contrast remains above the existing thresholds.

## Live Patina palette

Popup uses the user's Live Patina / High Current color family. Dark mode has deep petrol `#03262a`, enamel `#094044`, lifted petrol `#084c4f`, warm ivory `#f4ebd6`, pale sage `#c0d1c3`, current `#44d6c0` and electric-mint focus `#86fae3`. Light mode is a derived companion: ivory canvas, pale mineral surfaces, petrol text and deeper teal actions. Error foregrounds and control boundaries are adjusted within the family for contrast; copper marks failures and green marks confirmed completion. Selection uses electric mint with deep ink. Brass is reserved for meaningful attention, not decoration.

The host still selects light/dark mode and can supply fonts. Popup owns its color tokens so host colors cannot replace the palette or break paired foreground/background contrast. Existing density, geometry, expanded branches and interaction behavior remain unchanged. Browser contrast checks cover text, support text, active/hover actions, selected surfaces, selection, focus, control edges, error panels and success colors in both themes. Screenshot fixtures remain qualitative evidence, not proof of behavior in every host.

## Authoring contract

`popup({ elements, title?, submit?, notes? })` accepts the classic element-as-key DSL. The tool's description, field annotations, recursive schema and examples are in `src/authoring`; these are product behavior, not generated documentation. The exact schema is preserved in `tools/list`.

Author roughly the next three useful conversational turns, stopping where new evidence is needed. Use Markdown for analysis, structured choices for structurable decisions, and text inputs for editable wording. `select` represents exclusive alternatives; `multi` lets compatible approaches coexist. Exploration and understanding are useful goals, even without a concrete action. Recommendation reasons appear independently of selection; leave exploratory choices initially empty. Omit redundant titles.

Prefer stable options `{ id, label, description?, recommendation?, children? }`: `id` is the wire value, `label` is display text, and inline `children` reveals answer-specific follow-ups for select/multi. Strings and legacy `{ value, description?, recommendation? }` remain valid; `because` aliases `description`. Stable and legacy options may mix if wire values are unique. Defaults, conditions and keyed branches use stable IDs or legacy values. Do not define both inline children and a keyed branch for one option; rank options cannot have children. Stable IDs that match widget properties require inline children rather than ambiguous keyed branches.

Place shared follow-ups outside branches in a group with `when: "any(selected(choices, a), selected(choices, b))"`. Use `{ disclosure: "Inspect proposal", elements: [...], open?: boolean }` for visually optional detail. A closed disclosure still contributes its visible child answers; `when` controls actual visibility. Every select/multi includes free-text Other automatically; widgets using stable options return `_other` for it. Reserve `_option_labels` alongside the other result metadata IDs.

A topic selector followed by “Tell me more” spends an interaction without anticipating much. Each branch should contribute a useful response or expose a consequential choice. The CI example supplies fictional evidence, recommends caching without selecting it, offers structured isolation alternatives, and shares an evaluation question outside the branches. Its disclosure holds supporting cache detail and editable change-summary wording. The full executable definition is `ciWorkedExample` in `src/authoring/index.ts`; the description and schema both include the evidence.

The tool returns `awaiting_input` with a `formId` and the definition immediately. The iframe locally evaluates branches and collects answers. On Submit or Cancel, it sends a user message:

```json
{
  "type": "popup_result",
  "formId": "…",
  "submissionId": "…",
  "result": {
    "status": "completed",
    "button": "submit",
    "changes": ["parallel"],
    "db_isolation": "investigate",
    "next_goal": "understand",
    "_labels": {
      "changes": "Which directions should we explore together?",
      "db_isolation": "What would help next?",
      "next_goal": "What should the next conversation establish?"
    },
    "_option_labels": {
      "changes": { "parallel": "Parallelize integration tests" },
      "db_isolation": { "investigate": "Understand isolation risks" },
      "next_goal": { "understand": "Clarify risks and tradeoffs" }
    }
  }
}
```

The form locks while sending. An explicit retry reuses the frozen result and submission ID. Acknowledgment means the host accepted the message, not that the agent has completed its continuation. Submitted forms cannot be edited. Repeated submission IDs represent duplicate delivery and must not repeat the downstream action. Cancellation returns `{ "status": "cancelled" }` without answers.

## Compatibility and deliberate refinements

All 22 classic JSON examples in the original repository are checked against both the compiler and the advertised schema. The semantic-routing example is deliberately excluded. Supported widgets are text, Markdown, sliders, checkboxes, text inputs, single/multiple choices, ranking, groups, visual disclosures, and linear pages. `nodes`, `page.next`, Jev routing and route outputs are excluded.

Hidden branches retain draft values but contribute neither condition values nor submitted answers. Unvisited pages also contribute no answers. Conditions are parsed once without evaluating JavaScript; dependency cycles are rejected with a path, replacing the old eight-pass visibility limit. Changes recompute only dependent nodes. Validation checks types, IDs, option references, generated Other inputs, bounded nesting and definition size before displaying a form.

Corrected quirks include repeated negation and escaped condition strings, schema/parser inconsistencies, Other injection inside pages, and automatic free text even when authors supply an Other branch. Explicit slider defaults are preserved, including off-grid values; implicit midpoints and user slider changes snap to the configured step. Markdown cannot load external media or execute HTML scripts; external links go through the host.

`evals/` contains fixed authoring/continuation scenarios, a rubric and a clearly labeled Sol self-sample. A subsequent comparison generated sixteen independent candidates; all compiled, while blinded feedback identified verbosity and repeated editable summaries in the first revised guidance. The worked example was then shortened, with one grounded branch open initially and investigation choices replacing repository-fact questions. Four follow-up samples also validate, with some verbosity remaining. These observations do not establish that the final guidance outperforms the baseline. Frozen baseline, first-candidate and final bundles preserve those historical compared versions; they predate stable options and disclosures and are not evaluations of the current authoring guidance; see the [evaluation report](evals/comparison/README.md) for its methods and limits. `npm run eval:comparison` revalidates the artifacts and eighty result replays without making new model calls.

## Dependencies and deployment

The application uses Solid 1.x, Vite 8, Cloudflare Agents and MCP Apps 2.x. MCP packages are pinned to patched 2.3.1 releases, with scoped overrides for Agents' older exact peers. The required legacy peer is patched to 1.32.1. Keep these overrides covered by protocol tests when updating dependencies.

Only an ASSETS binding is needed; no database, Durable Object, model API or secret is required. `/health` reports availability. Observability records operational request metadata, not form contents or answers. Wrangler deploys the name in `wrangler.jsonc` to the selected Cloudflare account.
