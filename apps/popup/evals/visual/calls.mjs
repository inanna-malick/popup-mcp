// Authored as actual popup calls: the visible screen must earn the user's attention.
export const scenarios = [
  {
    id: "investigate",
    context:
      "CI is slow. We know downloads repeat and integration tests share a database. Offer a bounded, configurable investigation rather than asking the user to reconstruct a plan.",
    call: {
      name: "popup",
      arguments: {
        title: "Find the expensive parts",
        submit: "Investigate these paths",
        elements: [
          {
            markdown:
              "**Start with measurement.** Downloads repeat across jobs; integration tests share a database. Both are useful leads, with different risks.",
          },
          {
            multi: "Investigation scope",
            id: "scope",
            options: [
              {
                id: "cache",
                label: "Dependency cache",
                description:
                  "Compare cold and warm installs without changing resolution.",
                recommendation: "The clearest low-risk lead.",
                children: [
                  {
                    slider: "Sample runs",
                    id: "runs",
                    min: 2,
                    max: 20,
                    default: 5,
                  },
                  {
                    check: "Include a cold-cache baseline",
                    id: "baseline",
                    default: true,
                  },
                ],
              },
              {
                id: "parallel",
                label: "Test parallelism",
                description: "Inspect isolation before adding workers.",
                children: [
                  {
                    slider: "Worker ceiling",
                    id: "workers",
                    min: 1,
                    max: 16,
                    default: 4,
                  },
                  {
                    slider: "Budget · minutes",
                    id: "budget",
                    min: 5,
                    max: 60,
                    default: 15,
                  },
                  {
                    input: "Test command",
                    id: "command",
                    default: "npm run test:integration",
                  },
                  {
                    select: "First isolation step",
                    id: "isolation",
                    options: [
                      {
                        id: "inspect",
                        label: "Inspect shared fixtures",
                        recommendation:
                          "Find global state before designing around it.",
                      },
                      {
                        id: "isolate",
                        label: "Prototype a database per worker",
                      },
                    ],
                  },
                ],
              },
              {
                id: "matrix",
                label: "Platform matrix",
                description:
                  "Identify overlap while preserving release coverage.",
              },
            ],
          },
          {
            group: "Capture once",
            when: "count(scope) > 0",
            elements: [
              { check: "Per-job timings", id: "timings", default: true },
              { check: "Cache hit rates", id: "hits", default: true },
              { check: "Runner cost", id: "cost" },
            ],
          },
          {
            disclosure: "What this investigation will establish",
            elements: [
              {
                markdown:
                  "A comparison of time saved, runner cost, and isolation risk. **Implementation comes after the measurements.**",
              },
            ],
          },
        ],
      },
    },
    explore: [
      { role: "checkbox", name: "Dependency cache" },
      { role: "checkbox", name: "Test parallelism" },
      { role: "radio", name: "Inspect shared fixtures" },
    ],
  },
  {
    id: "story",
    context:
      "An author wants a stranger opening without losing a clear narrative thread. Offer compatible editorial moves and the next consequential choices, including editable wording.",
    call: {
      name: "popup",
      arguments: {
        title: "Make the opening stranger",
        submit: "Draft this version",
        elements: [
          {
            markdown:
              "Keep the narrator's objective clear. Let the **world** become uncertain around it.",
          },
          {
            multi: "Editorial moves",
            id: "moves",
            options: [
              {
                id: "object",
                label: "One impossible object",
                description:
                  "An ordinary errand acquires a precise, inexplicable detail.",
                recommendation:
                  "A small disturbance can carry the whole scene.",
                children: [
                  {
                    select: "The disturbance",
                    id: "object_kind",
                    options: [
                      {
                        id: "map",
                        label: "A map of a room that does not exist",
                      },
                      {
                        id: "letter",
                        label: "A letter in the narrator's own handwriting",
                      },
                    ],
                  },
                  {
                    slider: "Reveal by paragraph",
                    id: "reveal",
                    min: 1,
                    max: 8,
                    default: 3,
                  },
                  {
                    input: "First sentence",
                    id: "opening",
                    rows: 3,
                    default:
                      "The map arrived on Tuesday, showing a room our house had never contained.",
                  },
                ],
              },
              {
                id: "voice",
                label: "A narrator who knows too much",
                description:
                  "Casual certainty about something the reader cannot explain.",
                children: [
                  {
                    select: "Distance from the reader",
                    id: "distance",
                    options: [
                      { id: "intimate", label: "Intimate and complicit" },
                      { id: "formal", label: "Formal and faintly evasive" },
                    ],
                  },
                  {
                    check: "Keep the narrator's immediate goal explicit",
                    id: "goal",
                    default: true,
                  },
                ],
              },
            ],
          },
          {
            markdown:
              "With both moves, let the object carry the mystery and keep the voice restrained. Two competing sources of strangeness would muddy the opening.",
            when: "selected(moves, object) && selected(moves, voice)",
          },
          {
            disclosure: "Inspect the proposed opening",
            elements: [
              {
                markdown:
                  "The map arrived on Tuesday, showing a room our house had never contained. I put it beside the unpaid bills. My sister was coming for dinner, and there were still practical things to hide.",
              },
              {
                input: "A detail worth keeping",
                id: "keep",
                default: "The practical concern about dinner",
              },
            ],
          },
        ],
      },
    },
    explore: [
      { role: "checkbox", name: "One impossible object" },
      { role: "radio", name: "A map of a room that does not exist" },
      { role: "checkbox", name: "A narrator who knows too much" },
      { role: "radio", name: "Formal and faintly evasive" },
      { summary: "Inspect the proposed opening" },
    ],
  },
  {
    id: "release",
    context:
      "An agent has a change ready and needs to expose release tradeoffs. Use a short control panel with ranking, sliders, constraints, and a meaningful escape hatch.",
    call: {
      name: "popup",
      arguments: {
        title: "Choose a release shape",
        submit: "Prepare the rollout",
        elements: [
          {
            select: "Rollout",
            id: "rollout",
            options: [
              {
                id: "canary",
                label: "Small canary first",
                description:
                  "Gather real traffic evidence before broadening exposure.",
                recommendation:
                  "Makes rollback cheap while uncertainty is high.",
                children: [
                  {
                    slider: "Initial traffic · %",
                    id: "traffic",
                    min: 1,
                    max: 25,
                    default: 5,
                  },
                  {
                    slider: "Observation · hours",
                    id: "hours",
                    min: 1,
                    max: 24,
                    default: 4,
                  },
                  {
                    check: "Stop on elevated error rate",
                    id: "errors",
                    default: true,
                  },
                  {
                    check: "Stop on latency regression",
                    id: "latency",
                    default: true,
                  },
                ],
              },
              {
                id: "internal",
                label: "Internal users only",
                description:
                  "Check workflow quality before collecting production evidence.",
              },
              {
                id: "wait",
                label: "Hold for more evidence",
                children: [{ input: "What is missing?", id: "missing" }],
              },
            ],
          },
          {
            rank: "Review priorities",
            id: "priorities",
            options: [
              { id: "recovery", label: "Fast recovery" },
              { id: "learning", label: "Quality of evidence" },
              { id: "reach", label: "Time to broad availability" },
            ],
          },
          {
            disclosure: "Operational details",
            elements: [
              { input: "Rollback owner", id: "owner", default: "Release lead" },
              {
                input: "Decision record",
                id: "record",
                rows: 2,
                default:
                  "Broaden only after error rate and latency remain stable throughout the observation window.",
              },
            ],
          },
        ],
      },
    },
    explore: [
      { role: "radio", name: "Small canary first" },
      { role: "button", name: "Fast recovery" },
      { role: "button", name: "Quality of evidence" },
      { summary: "Operational details" },
    ],
  },
];

scenarios.push({
  id: "workspace",
  context:
    "Set a bounded repository investigation: execution, scope, and evidence are related control groups that can be scanned side by side.",
  call: {
    name: "popup",
    arguments: {
      title: "Set the investigation boundaries",
      submit: "Start with these settings",
      elements: [
        {
          markdown:
            "I can trace the slow path and compare a few targeted changes. These settings define the investigation, before any implementation work.",
        },
        {
          group: "Execution",
          elements: [
            {
              slider: "Time budget · minutes",
              id: "time",
              min: 5,
              max: 60,
              default: 20,
            },
            {
              slider: "Parallel workers",
              id: "workers",
              min: 1,
              max: 8,
              default: 2,
            },
            {
              input: "Test command",
              id: "command",
              default: "npm run test:integration",
            },
            {
              check: "Keep the working tree unchanged",
              id: "readonly",
              default: true,
            },
          ],
        },
        {
          group: "Evidence",
          elements: [
            {
              multi: "Capture",
              id: "capture",
              options: ["Timings", "Memory", "Cache hits", "Runner cost"],
            },
            { check: "Repeat noisy measurements", id: "repeat", default: true },
            {
              check: "Compare with the current baseline",
              id: "baseline",
              default: true,
            },
          ],
        },
        {
          multi: "Platforms",
          id: "platforms",
          options: ["Linux", "macOS", "Windows"],
        },
        {
          disclosure: "What comes back",
          elements: [
            {
              markdown:
                "A short account of the bottleneck, measurements supporting it, and the smallest change worth testing next.",
            },
          ],
        },
      ],
    },
  },
  explore: [
    { role: "checkbox", name: "Timings" },
    { role: "checkbox", name: "Cache hits" },
    { role: "checkbox", name: "Linux" },
    { role: "checkbox", name: "macOS" },
  ],
});
