/** Authored contrast pairs, not model samples; contexts are comparison/tasks.json. */
const cases = [
  [
    "ci",
    "Four jobs repeat a two-minute install; the eleven-minute integration suite shares a database. Preserve release coverage.",
    "Cache dependency downloads",
    "Measure cold and warm installs; keep immutable dependency resolution.",
    "Investigate isolated parallel tests",
    "Inspect database naming and cleanup before benchmarking workers; runner spend may offset gains.",
    "Benchmark caching and parallelism separately so faster installs do not conceal database contention.",
  ],
  [
    "performance",
    "Repeated queries take 620 ms of the 900 ms response; rendering 2,000 rows adds 350 ms. Billing edits must stay fresh; search must cover all rows.",
    "Investigate query batching",
    "Inspect query plans before changing SQL. Summary reporting may lag 30 seconds; billing edits may not.",
    "Explore fewer rendered rows",
    "Compare pagination and virtualization with keyboard navigation while searching the complete data set.",
    "Measure server and browser time separately so query improvements do not mask navigation regressions.",
  ],
  [
    "exploration",
    "A benevolent public memory archive can create intimate political tensions through correction and borrowing, without a final plot commitment.",
    "Dispute an official memory",
    "An archivist preserves two truthful conflicting accounts. A family must live with both, while the public interprets them.",
    "Borrow a loved one's memory",
    "A partner borrows a happy afternoon and develops affection for someone they never met. Returning it protects autonomy but feels like rejection.",
    "A private relationship becomes testimony in a correction dispute: the city's legitimate interest competes with the couple's intimacy.",
  ],
  [
    "evidence",
    "p95 crossed ten seconds after deployment; timing alone does not identify the cause. Telemetry reads are available; production changes need explicit authorization.",
    "Investigate the regression",
    "Compare traces and errors before and after deployment by route and workload. The earlier downstream incident is a hypothesis, not evidence.",
    "Prepare conditional mitigation",
    "If failures trace to the release, prepare rollback benefits and risks. If traffic explains the alert, reassess before proposing production changes.",
    "Diagnosis and mitigation criteria use the same evidence; preparing an option does not authorize executing it.",
  ],
];
export const densityPairs = cases.map(
  ([id, contribution, a, reasoningA, b, reasoningB, interaction]) => {
    const definition = (overlong: boolean) => ({
      elements: [
        ...(overlong
          ? [
              {
                markdown:
                  "Let's explore the useful possibilities together before deciding how to proceed.",
              },
            ]
          : []),
        { markdown: contribution },
        ...(overlong ? [{ markdown: `To recap: ${contribution}` }] : []),
        {
          multi: "Which directions should we explore?",
          id: "directions",
          options: [
            {
              id: "a",
              label: a,
              description: reasoningA,
              children: overlong
                ? [{ markdown: contribution }, { markdown: reasoningA }]
                : [],
            },
            {
              id: "b",
              label: b,
              description: reasoningB,
              children: overlong
                ? [{ markdown: contribution }, { markdown: reasoningB }]
                : [],
            },
          ],
        },
        {
          group: "Next step",
          when: "any(selected(directions, a), selected(directions, b))",
          elements: [
            {
              select: "What should the next conversation establish?",
              id: "next",
              options: ["Explore consequences", "Prepare a concrete proposal"],
            },
          ],
        },
        {
          markdown: interaction,
          when: "selected(directions, a) && selected(directions, b)",
        },
      ],
    });
    return {
      id,
      substantiveClaims: [contribution, reasoningA, reasoningB, interaction],
      concise: definition(false),
      overlong: definition(true),
    };
  },
);
