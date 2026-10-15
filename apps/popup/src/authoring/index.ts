/** Plain recursive JSON Schema: retain descriptions and examples as model-facing assets. */
const whenDescription =
  "Visibility over visible values only: bare id tests truthiness; selected(id, value) tests an option; count(id) > N counts selections; any(...)/all(...), &&, ||, !, comparisons and parentheses compose conditions. Negative numbers are allowed. Quote option text containing a closing parenthesis. Hidden values read as unset (false, unselected, count 0). References must exist and dependencies must be acyclic, including ancestor/descendant dependencies.";
const id = {
  type: "string",
  maxLength: 100000,
  pattern: "^[a-zA-Z_][a-zA-Z0-9_]*$",
  description:
    "Globally unique ASCII identifier; otherwise derived from label. Reserved: status, button, error, _notes, _labels, _option_labels, _route, __proto__, constructor, prototype. Avoid collisions with generated <id>_other_text fields.",
};
const when = {
  type: "string",
  maxLength: 100000,
  description: whenDescription,
};
const children = { $ref: "#/properties/elements" };
const optionAnnotations = {
  description: {
    type: "string",
    maxLength: 100000,
    description: "Consequence, tradeoff or assumption.",
  },
  because: {
    type: "string",
    maxLength: 100000,
    description: "Alias for description; use only one.",
  },
  recommendation: {
    type: "string",
    minLength: 1,
    pattern: "\\S",
    maxLength: 100000,
    description: "Visible reason to recommend this option; does not select it.",
  },
};
const option = (branching: boolean) => ({
  oneOf: [
    { type: "string", maxLength: 100000 },
    {
      type: "object",
      properties: {
        value: { type: "string", maxLength: 100000 },
        ...optionAnnotations,
      },
      required: ["value"],
      not: { required: ["description", "because"] },
      additionalProperties: false,
    },
    {
      type: "object",
      properties: {
        id: {
          type: "string",
          maxLength: 100000,
          pattern: "^[a-zA-Z_][a-zA-Z0-9_]*$",
          description:
            "Stable wire value used by defaults, selected() and returned answers; unique within this widget.",
        },
        label: {
          type: "string",
          minLength: 1,
          pattern: "\\S",
          maxLength: 100000,
          description:
            "Display text; may change without changing the option identity.",
        },
        ...optionAnnotations,
        ...(branching
          ? {
              children: {
                ...children,
                description:
                  "Inline answer-specific follow-ups. Do not also supply a keyed branch for this id.",
              },
            }
          : {}),
      },
      required: ["id", "label"],
      not: { required: ["description", "because"] },
      additionalProperties: false,
    },
  ],
});
const options = (minimum: number, description: string, branching = true) => ({
  type: "array",
  minItems: minimum,
  maxItems: 200,
  items: option(branching),
  description,
});
const element = (
  name: string,
  description: string,
  properties: Record<string, unknown>,
  required: string[] = [],
  additionalProperties: unknown = false,
) => ({
  type: "object",
  description,
  properties: {
    [name]: {
      type: "string",
      maxLength: 100000,
      description:
        "Conversational content: analysis, proposal, question or context appropriate to this element.",
    },
    id,
    when,
    ...properties,
  },
  required: [name, ...required],
  additionalProperties,
});

/** Fictional evidence is supplied alongside this example in popupDescription. */
export const ciWorkedExample = {
  submit: "Continue with these directions",
  elements: [
    {
      markdown:
        "Repeated four-minute installs make caching a useful lead. The eighteen-minute integration suite shares a database, so parallelism needs isolation evidence. Platform coverage and release commitments need inspection before exclusions.",
    },
    {
      multi: "Which directions should we explore together?",
      id: "changes",
      options: [
        {
          id: "cache",
          label: "Cache dependencies",
          recommendation:
            "Repeated downloads make this the clearest low-risk lead.",
          children: [
            {
              select: "On a lockfile change?",
              id: "cache_miss",
              options: [
                {
                  id: "rebuild",
                  label: "Rebuild the cache",
                  description: "Simple invalidation; first run stays cold.",
                  recommendation:
                    "Start simple before adding fallback complexity.",
                },
                {
                  id: "restore",
                  label: "Restore a nearby cache",
                  description:
                    "Potentially faster; verify stale downloads are harmless.",
                  children: [
                    {
                      check: "Benchmark fallback before adopting it",
                      id: "benchmark_restore",
                    },
                  ],
                },
              ],
            },
            {
              input: "Suggested change-summary wording",
              id: "cache_summary",
              default:
                "Cache dependency downloads without changing dependency resolution.",
            },
            {
              disclosure: "Inspect the cache proposal",
              elements: [
                {
                  markdown:
                    "Cache the download store by OS, runtime and lockfile hash; retain immutable installs. I’ll inspect the install command before choosing paths, since native rebuilds may need a different intervention.",
                },
              ],
            },
          ],
        },
        {
          id: "parallel",
          label: "Parallelize integration tests",
          description: "Target the longest job; inspect shared state first.",
          children: [
            {
              select: "What would help next?",
              id: "db_isolation",
              options: [
                {
                  id: "investigate",
                  label: "Understand isolation risks",
                  recommendation:
                    "Inspect fixtures, database naming and cleanup before promising concurrency.",
                },
                {
                  id: "proposal",
                  label: "Explore an isolation design",
                  children: [
                    {
                      select: "Design to compare",
                      id: "isolation_design",
                      options: [
                        {
                          id: "database",
                          label: "Database per worker",
                          description:
                            "Strong separation; migrations and startup cost multiply.",
                        },
                        {
                          id: "schema",
                          label: "Schema per worker",
                          description:
                            "Cheaper setup; verify code has no database-global assumptions.",
                        },
                      ],
                    },
                    {
                      slider: "Workers to benchmark after validation",
                      id: "workers",
                      min: 2,
                      max: 8,
                      default: 2,
                    },
                  ],
                },
              ],
            },
          ],
        },
        {
          id: "coverage",
          label: "Reconsider platform coverage",
          description: "Explore PR cost versus platform-specific confidence.",
          children: [
            {
              markdown:
                "I’ll inspect native code, filesystem-sensitive tests and support policy before proposing exclusions.",
            },
            {
              select: "Scope to explore",
              id: "coverage",
              options: [
                { id: "audit", label: "Understand current coverage" },
                {
                  id: "pr",
                  label: "Compare smaller PR matrices",
                  description:
                    "Preserve the full release matrix while evaluating representative PR checks.",
                },
              ],
            },
          ],
        },
      ],
    },
    {
      group: "Shared evaluation",
      when: "any(selected(changes, cache), selected(changes, parallel), selected(changes, coverage))",
      elements: [
        {
          select: "What should the next conversation establish?",
          id: "next_goal",
          options: [
            { id: "understand", label: "Clarify risks and tradeoffs" },
            { id: "measure", label: "Compare measured outcomes" },
            { id: "change", label: "Prepare a concrete change proposal" },
          ],
        },
        {
          markdown:
            "Benchmark caching and parallelism separately before combining them, so faster installs do not conceal database contention.",
          when: "selected(changes, cache) && selected(changes, parallel)",
        },
      ],
    },
  ],
};

export const popupInputSchema = {
  type: "object" as const,
  properties: {
    title: {
      type: "string",
      maxLength: 100000,
      description:
        "Optional short context heading; omit when the conversation already supplies context.",
    },
    submit: {
      type: "string",
      maxLength: 100000,
      description:
        "Submission label, default Submit. Use a verb matching the requested consequence.",
    },
    notes: {
      type: "boolean",
      description:
        "Show optional Add a note, default true. Keep enabled to catch unforeseen answers and corrections.",
    },
    elements: {
      type: "array",
      description:
        "Conversational elements in display order: explain, propose and expose useful continuations. The widget key carries its label.",
      items: {
        oneOf: [
          element("text", "Static text.", {}),
          element("markdown", "Markdown framing or context.", {}),
          element(
            "slider",
            "Numeric slider; min < max, step > 0, default within bounds.",
            {
              min: { type: "number" },
              max: { type: "number" },
              step: {
                type: "number",
                exclusiveMinimum: 0,
                description: "Default 1.",
              },
              default: {
                type: "number",
                description:
                  "Implicit midpoint snapped to step; explicit in-range values are preserved.",
              },
            },
            ["min", "max"],
          ),
          element(
            "check",
            "Checkbox; checked-branch follow-ups belong in reveals.",
            {
              default: { type: "boolean", description: "Default false." },
              reveals: { ...children, description: "Shown while checked." },
            },
          ),
          element("input", "Free text input.", {
            placeholder: { type: "string", maxLength: 100000 },
            default: {
              type: "string",
              maxLength: 100000,
              description: "Proposed text the user can edit.",
            },
            rows: {
              type: "integer",
              minimum: 1,
              maximum: 100,
              description: "Multiline height.",
            },
          }),
          element(
            "select",
            "Single choice. Prefer stable id/label options with inline children; legacy strings/value options remain supported.",
            {
              options: options(
                1,
                "Other (please specify) and its text input are automatic; do not author them.",
              ),
              default: {
                type: "string",
                maxLength: 100000,
                description:
                  "Initial wire value: stable option id or legacy option text. Recommendation reasons do not select options.",
              },
              style: {
                type: "string",
                maxLength: 100000,
                enum: ["radio", "dropdown"],
                description:
                  "Automatic: radio up to six options, otherwise dropdown.",
              },
              reveals: {
                ...children,
                description: "Shown once any option is selected.",
              },
            },
            ["options"],
            {
              ...children,
              description:
                "Branch key must match a stable option id or legacy value; cannot coexist with that option’s inline children.",
            },
          ),
          element(
            "multi",
            "Compatible choices; each selected option reveals inline children or a keyed branch independently.",
            {
              options: options(
                1,
                "Other (please specify) and its text input are automatic.",
              ),
              default: {
                oneOf: [
                  { type: "string", maxLength: 100000 },
                  {
                    type: "array",
                    items: { type: "string", maxLength: 100000 },
                  },
                ],
                description:
                  "Initial wire values: stable ids or legacy texts; a string is a one-item array. Omit for an initially empty exploration.",
              },
              reveals: {
                ...children,
                description: "Shown once any option is selected.",
              },
            },
            ["options"],
            {
              ...children,
              description:
                "Branch key must match a stable option id or legacy value; cannot coexist with that option’s inline children.",
            },
          ),
          element(
            "rank",
            "Click to add priorities, then reorder ranked items directly. Partial rankings are valid; unranked items are omitted.",
            {
              options: options(
                2,
                "At least two options; no automatic Other or children.",
                false,
              ),
            },
            ["options"],
          ),
          element(
            "group",
            "Visual grouping of related elements.",
            { elements: children },
            ["elements"],
          ),
          element(
            "disclosure",
            "Visual-only expandable detail. Closing it preserves selection, visibility and submitted child answers; when still controls visibility.",
            {
              elements: children,
              open: {
                type: "boolean",
                description: "Initially expanded; default false.",
              },
            },
            ["elements"],
          ),
          element(
            "page",
            "Linear wizard step. Top-level only; all top-level elements must be pages when used. Back/Next, one terminal Submit; when can skip a step. No next edges or semantic routing.",
            { elements: children },
            ["elements"],
          ),
        ],
      },
    },
  },
  required: ["elements"],
  additionalProperties: false,
  examples: [ciWorkedExample],
};

export const popupDescription = `Continue the conversation in an interactive message. Partially evaluate roughly the next three useful turns: use what is already known to explain the situation, offer consequential proposals, and prepare the follow-ups each plausible answer would invite. Start with a concise grounded contribution: the observation, its implication, and the uncertainty that changes the next action. Offer consequential alternatives rather than a catalog of topics. Size the initial surface by the decision at hand; put answer-specific reasoning and proposals in the relevant branch. Active branches always stay expanded, including simultaneous multi selections: reduce oversized surfaces through authoring, never by relying on branch collapse. Preserve useful causal reasoning, tradeoffs and expressive possibilities; no fixed word, option, pixel or nesting budget replaces judgment. Do not repeat a summary as an editable input unless editing it changes the next action. An investigation or an editable proposal is a successful outcome; stop when the next step needs new evidence rather than inventing that evidence or extending an exhaustive questionnaire.

Separate observations from assumptions. Ground recommendations in the conversation or inspected material; mark unknowns and use structured choices for structurable plans and text inputs for editable wording. Recommendations are visible reasons, independent of default selection; leave exploratory choices initially empty. Use defaults for established preferences or low-consequence editable proposals, not to turn a recommendation into agreement or pre-open an unchosen exploration. Use markdown/text for substantive analysis, select for mutually exclusive directions and multi for compatible approaches the user may explore together. Prefer stable options {id,label,recommendation?,children?}; put answer-specific continuations in children; each selected multi branch stays visible alongside the others. Prefer reveals for checked or selection-wide continuations, then when for cross-branch interactions. Roughly three turns is a foresight horizon, not a required depth. Inspect repository facts yourself; ask the user about consequential choices, not facts you can retrieve. Defaults are proposals until Submit.

DENSITY CONTRAST (same reasoning): overlong framing says "The trace shows 620 ms of repeated queries within a 900 ms response. Repeated database queries contribute substantially to the response. We should investigate these repeated queries before making changes." A concise contribution says "Repeated queries account for 620 ms of the 900 ms response; inspect a query plan before choosing batching." Both retain evidence and the next investigation; the latter removes restatement. Keep distinct constraints such as fresh billing edits and full-data search wherever they affect the proposal. For creative exploration, preserve the concrete possibility and its tension rather than reducing it to a topic label.

FAILURE CASE: a topic selector followed by "Tell me more" delegates the thinking back to the user. Contribute a hypothesis, tradeoff or proposal at the selected branch instead; when evidence is missing, offer the next investigation.

WORKED EXAMPLE: the following CI timings are fictional supplied evidence, not facts about your repository. Notice the analysis, simultaneously explorable changes, stable identities, unselected recommendations, structured exploration, optional detail and shared follow-ups outside the branches. Transfer that structure rather than copying its assumptions:
${JSON.stringify(ciWorkedExample)}

Widgets: text, markdown, slider, check, input, select, multi, rank, group, disclosure and linear page. A disclosure {disclosure:"Inspect proposal",elements:[...],open?:boolean} collapses detail visually; closing it does not deselect or omit its child answers. when still hides answers. title is optional; omit redundant headings. Give options decision-relevant descriptions (because is an alias). multi.default accepts an option string or array. Stable options use {id,label,description?/because?,recommendation?,children?}; children belongs only to select/multi. Legacy strings or {value,description?/because?,recommendation?} still work and may mix with stable options if wire values are unique. Defaults, selected() and keyed branches use stable ids or legacy values. Prefer inline children; do not also author a keyed branch for the same option. Shared follow-ups belong outside branches in a group with when: any(selected(...),selected(...)). State common analysis once; each branch adds only the reasoning or decision that differs. Cross-branch analysis should explain an interaction, not repeat both branches. Use disclosure only for supplementary reference detail, never to hide the contribution, active continuation or a consequential question. Remove redundant framing, repeated caveats, decorative headings and requests for facts already available; retain evidence, uncertainty and consequences that can change the answer. reveals appears while checked or once a selection exists. Conditions use bare ids, selected(id, value), count(id), any/all, comparisons, && || ! and parentheses; quote values containing a closing parenthesis. Hidden values read as unset even with defaults. References must exist; cycles including ancestor/descendant dependencies are rejected. If pages are needed, every top-level element is a page with Back/Next and one final Submit; page.when can skip steps.

Keep notes enabled. Every select/multi supplies Other (please specify) with free text automatically. Widgets using stable options return _other for that option; if authoring _other explicitly, use that exact label. Notes and unexpected answers can change the problem representation; explicit corrections override conflicting widgets. Ask only what still blocks progress. Use unique ASCII ids; avoid status, button, error, _notes, _labels, _option_labels, _route, __proto__, constructor, prototype and generated <id>_other_text collisions. Legacy option values cannot collide with widget properties; stable ids can, but use inline children for those ids. Repair path-specific validation errors and retry. No page.next, conversation nodes, semantic routing, after_submit or intermediate submission exists.

Opening the tool returns awaiting_input with formId and definition, not a decision. Wait for the user message {"type":"popup_result","formId":"…","submissionId":"…","result":{…}}. Its terminal classic result is {"status":"completed","button":"submit","<id>":<value>,"_labels":{…},"_notes":"…"} or {"status":"cancelled"}. A repeated submissionId is duplicate delivery: do not repeat the action. Cancellation stops this question; do not reopen automatically. There is no backend answer storage.

Interpret the submitted evidence: absent keys were hidden or on unvisited pages; input "" is blank, select null unchosen, multi/rank [] empty, check false unchecked. Select returns a stable id or legacy option text; multi/rank return their wire values in arrays (rank in priority order). _option_labels[fieldId] maps selected stable ids to their displayed labels; legacy-only results remain unchanged; slider returns a number; Other adds <id>_other_text. _labels explains ids and _notes adds instructions. Resume the original task from the actual answer, including corrections, without mistaking the opening acknowledgment or an unsubmitted default for agreement.`;
