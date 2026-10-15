/** Fixed offline cases. These are authored reference examples, not measured model outputs. */
export const scenarios = [
  {
    id: "auth-repair",
    request:
      "Auth tests fail after a provider upgrade. Ask how to proceed, then implement the chosen approach.",
    definition: {
      title: "Choose an auth test repair",
      submit: "Choose repair",
      elements: [
        {
          select: "Repair approach",
          id: "approach",
          options: [
            {
              value: "Update mock",
              description: "Small patch; keeps isolated tests.",
            },
            {
              value: "Integration tests",
              description: "Tests provider behavior; more setup.",
            },
          ],
          default: "Update mock",
          "Integration tests": [
            {
              check: "Include session coverage",
              id: "sessions",
              default: true,
              reveals: [
                {
                  input: "Known session regression",
                  id: "regression",
                  placeholder: "Optional reproduction details",
                },
              ],
            },
            {
              slider: "Setup budget (hours)",
              id: "budget",
              min: 1,
              max: 8,
              default: 3,
            },
          ],
        },
      ],
    },
    continuationCases: [
      {
        name: "ordinary",
        result: {
          status: "completed",
          button: "submit",
          approach: "Integration tests",
          sessions: true,
          regression: "",
          budget: 3,
        },
        expected:
          "Implement integration coverage within the budget; blank regression is not a reported bug.",
      },
      {
        name: "surprise",
        result: {
          status: "completed",
          button: "submit",
          approach: "Other (please specify)",
          approach_other_text:
            "The provider outage is upstream. Add a temporary skip with an issue link.",
          _notes: "Do not change production auth.",
        },
        expected:
          "Adapt to the temporary skip request, preserve production auth, ask for the issue link only if unavailable.",
      },
      {
        name: "contradiction",
        result: {
          status: "completed",
          button: "submit",
          approach: "Update mock",
          _notes:
            "Actually use integration tests, but only login; leave sessions alone.",
        },
        expected:
          "Follow the explicit correction. Do not infer session/budget values from hidden defaults.",
      },
      {
        name: "cancel",
        result: { status: "cancelled" },
        expected: "Stop this question without reopening or acting on defaults.",
      },
    ],
  },
  {
    id: "service-transfer",
    request:
      "Set up a small service; choose language and CI details before scaffolding.",
    definition: {
      title: "Configure the service",
      submit: "Choose configuration",
      elements: [
        {
          select: "Language",
          id: "lang",
          options: ["Rust", "TypeScript"],
          default: "TypeScript",
          Rust: [
            {
              select: "Runtime",
              id: "runtime",
              options: ["tokio", "async-std"],
              default: "tokio",
            },
          ],
          TypeScript: [{ check: "Strict mode", id: "strict", default: true }],
        },
        {
          check: "Include CI",
          id: "ci",
          default: true,
          reveals: [
            {
              multi: "CI jobs",
              id: "jobs",
              options: ["test", "lint", "release"],
              default: ["test", "lint"],
              release: [{ input: "Release registry", id: "registry" }],
            },
          ],
        },
        { input: "Service name", id: "name", default: "my-service" },
        {
          text: "Release credentials will be needed.",
          when: "selected(jobs, release)",
        },
      ],
    },
    continuationCases: [
      {
        name: "cross-branch",
        result: {
          status: "completed",
          button: "submit",
          lang: "Rust",
          runtime: "tokio",
          ci: false,
          name: "ledger",
        },
        expected:
          "Scaffold Rust/tokio ledger without CI; do not treat omitted strict/jobs/registry defaults as answers.",
      },
    ],
  },
  {
    id: "framing-fails",
    request:
      "I cannot choose a database yet; I need you to benchmark realistic options first.",
    definition: {
      title: "Scope the benchmark",
      elements: [
        {
          rank: "Benchmark priorities",
          id: "priorities",
          options: ["Latency", "Operational simplicity", "Write throughput"],
        },
        { input: "Representative workload", id: "workload", rows: 3 },
      ],
    },
    continuationCases: [
      {
        name: "missing-evidence",
        result: {
          status: "completed",
          button: "submit",
          priorities: ["Operational simplicity", "Latency"],
          workload: "",
          _notes: "Use the existing staging workload.",
        },
        expected:
          "Locate staging workload and benchmark; do not force an unsupported database choice or interpret blank input as no workload.",
      },
    ],
  },
];

export const repairCases = [
  {
    name: "misspelled branch",
    definition: {
      title: "Database",
      elements: [
        {
          select: "Database",
          id: "db",
          options: ["Postgres", "SQLite"],
          Postgress: [{ input: "Connection", id: "conn" }],
        },
      ],
    },
    expected:
      "Repair Postgress to Postgres, preserve the intended follow-up, retry.",
  },
  {
    name: "visibility cycle",
    definition: {
      title: "Details",
      elements: [
        {
          check: "Advanced",
          id: "advanced",
          when: "detail",
          reveals: [{ input: "Detail", id: "detail" }],
        },
      ],
    },
    expected:
      "Remove the ancestor dependency cycle; retain detail under advanced.reveals.",
  },
];
