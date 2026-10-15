import { describe, expect, it } from "vitest";
import {
  compileDefinition,
  computeVisibility,
  serializeResult,
  summaryRows,
  DefinitionError,
  createVisibilityEvaluator,
  labelToId,
  snapSliderValue,
} from "../src/core";
const form = (elements: unknown[]) =>
  compileDefinition({ title: "Test", elements });
const ids = (f: ReturnType<typeof form>, answers = {}) =>
  f.all
    .filter((n) => computeVisibility(f, answers).has(n.key))
    .map((n) => n.id)
    .filter(Boolean);
describe("classic form compilation", () => {
  it("compiles all widgets, aliases and injected Other through pages", () => {
    const f = form([
      {
        page: "Basics",
        elements: [
          { text: "Hello" },
          { markdown: "**Hi**" },
          {
            group: "Group",
            elements: [
              { slider: "CPU Usage", min: 0, max: 10, step: 0.5 },
              { input: "Name", default: "Ada", rows: 2 },
              { check: "Ready", default: true },
              { select: "Mode", options: [{ value: "A", because: "Easy" }] },
              { multi: "Features", options: ["X"], default: "X" },
              { rank: "Priority", options: ["One", "Two"] },
            ],
          },
        ],
      },
    ]);
    expect(f.defaults.cpu_usage).toBe(5);
    expect(f.defaults.features).toEqual(["X"]);
    expect(f.byId.get("mode")?.options[0].description).toBe("Easy");
    expect(f.byId.has("mode_other_text")).toBe(true);
    const r = serializeResult(
      f,
      f.defaults,
      " note ",
      new Set([f.nodes[0].key]),
    );
    expect(r).toMatchObject({
      status: "completed",
      button: "submit",
      name: "Ada",
      _notes: "note",
      _labels: { name: "Name" },
    });
    expect(r).not.toHaveProperty("mode_other_text");
  });
  it("retains hidden answers while excluding them from conditions and results", () => {
    const f = form([
      {
        check: "Enable",
        id: "enable",
        reveals: [{ input: "Secret", id: "secret" }],
      },
      { text: "Dependent", id: "dependent", when: 'secret == "yes"' },
      { input: "Negated", id: "negated", when: "!secret" },
    ]);
    const answers = { ...f.defaults, enable: false, secret: "yes" };
    expect(ids(f, answers)).not.toContain("dependent");
    expect(ids(f, answers)).toContain("negated");
    expect(serializeResult(f, answers, "")).not.toHaveProperty("secret");
    expect(answers.secret).toBe("yes");
  });
  it("evaluates forward references in dependency order", () => {
    const f = form([
      { input: "First", when: "second" },
      { check: "Second", default: true },
    ]);
    expect(ids(f, f.defaults)).toContain("first");
  });
  it("rejects direct, indirect and ancestor visibility cycles", () => {
    for (const elements of [
      [{ check: "A", when: "a" }],
      [
        { check: "A", when: "b" },
        { check: "B", when: "a" },
      ],
      [{ group: "Ancestor", when: "child", elements: [{ check: "Child" }] }],
    ])
      expect(() => form(elements)).toThrow(/cycle/);
  });
  it("handles negation, escaped strings and bare selected option values", () => {
    const f = form([
      { input: "Name", default: 'a"b' },
      { multi: "Choices", options: ["Long option"], default: ["Long option"] },
      { check: "Flag" },
      {
        text: "Visible",
        id: "visible",
        when: '!(flag || count(choices) == 0) && selected(choices, Long option) && name == "a\\\"b"',
      },
    ]);
    expect(ids(f, f.defaults)).toContain("visible");
  });
  it("uses empty array truthiness, strict equality and parent gates", () => {
    const f = form([
      { multi: "Choices", options: ["A"], reveals: [{ input: "Details" }] },
      { text: "Empty", id: "empty", when: "!choices" },
      { text: "Strict", id: "strict", when: "choices == false" },
    ]);
    expect(ids(f, f.defaults)).toContain("empty");
    expect(ids(f, f.defaults)).not.toContain("details");
    expect(ids(f, f.defaults)).not.toContain("strict");
  });
  it("preserves supplied Other branches and checks generated ID collisions", () => {
    const f = form([
      { select: "Pick", options: ["Other"], Other: [{ input: "Reason" }] },
    ]);
    expect(f.byId.has("pick_other_text")).toBe(true);
    expect(f.byId.has("reason")).toBe(true);
    expect(() =>
      form([
        { input: "Reserved", id: "pick_other_text" },
        { select: "Pick", options: ["A"] },
      ]),
    ).toThrow(/Duplicate ID/);
  });
  it("returns only visited, currently visible pages", () => {
    const f = form([
      { page: "One", elements: [{ input: "First" }] },
      { page: "Two", id: "two", elements: [{ input: "Second" }] },
    ]);
    expect(
      serializeResult(f, f.defaults, "", new Set([f.nodes[0].key])),
    ).not.toHaveProperty("second");
  });
  it("provides paths and rejects malformed or unsupported definitions", () => {
    for (const value of [
      { title: "X", nodes: [] },
      { title: "X", elements: [{ page: "P", elements: [], next: [] }] },
      { title: "X", elements: [{ check: "A", when: "missing" }] },
      { title: "X", elements: [{ check: "A", when: "true garbage" }] },
      { title: "X", elements: [{ input: "X", id: "status" }] },
      { title: "X", elements: [{ slider: "S", min: 3, max: 1 }] },
      { title: "X", elements: [{ input: "Same" }, { input: "Same" }] },
    ]) {
      try {
        compileDefinition(value);
        throw new Error("accepted");
      } catch (error) {
        expect(error).toBeInstanceOf(DefinitionError);
        expect((error as DefinitionError).issues[0].path).toMatch(/^\$/);
      }
    }
  });
  it("validates defaults and supplied answers without accepting unknown keys", () => {
    expect(() => form([{ multi: "M", options: ["A"], default: 3 }])).toThrow(
      /unique option values/,
    );
    const f = form([
      { slider: "S", min: 0, max: 10 },
      { rank: "R", options: ["A", "B"] },
    ]);
    expect(() => serializeResult(f, { unknown: true }, "")).toThrow(
      /Unknown answer/,
    );
    expect(() => serializeResult(f, { s: 11 }, "")).toThrow(/between/);
    expect(() => serializeResult(f, { r: ["A", "A"] }, "")).toThrow(/unique/);
  });
  it("preserves a cleared select with an authored default", () => {
    const f = form([{ select: "Choice", options: ["A"], default: "A" }]);
    expect(serializeResult(f, { choice: null }, "")).toHaveProperty(
      "choice",
      null,
    );
  });
  it("bounds definition depth and condition nesting", () => {
    let node: unknown = { input: "Leaf" };
    for (let i = 0; i < 35; i++) node = { group: `G${i}`, elements: [node] };
    expect(() => form([node])).toThrow(/Nesting/);
    expect(() =>
      form([{ text: "X", when: "(".repeat(70) + "true" + ")".repeat(70) }]),
    ).toThrow(/nesting/);
  });
  it("uses Rust label derivation and automatic select styles", () => {
    expect(labelToId("HTTPServer / URL.Value")).toBe("http_server_url_value");
    expect(labelToId("foo/bar")).toBe("foobar");
    const f = form([
      { select: "Small", options: ["1", "2", "3", "4", "5", "6"] },
      { select: "Large", options: ["1", "2", "3", "4", "5", "6", "7"] },
    ]);
    expect(f.byId.get("small")?.style).toBe("radio");
    expect(f.byId.get("large")?.style).toBe("dropdown");
    expect(() => form([{ slider: "S", min: 1, max: 1 }])).toThrow(/min < max/);
    expect(() => form([{ rank: "R", options: ["Only"] }])).toThrow(
      /two options/,
    );
  });
  it("suggests misspelled branch keys and selected options", () => {
    expect(() =>
      form([{ select: "Mode", options: ["Dark"], Drak: [] }]),
    ).toThrow(/did you mean "Dark"/);
    expect(() =>
      form([
        { select: "Mode", options: ["Dark"] },
        { text: "Info", when: "selected(mode, Drak)" },
      ]),
    ).toThrow(/did you mean "Dark"/);
  });
  it("matches full visibility through incremental dependency closures", () => {
    const f = form([
      { check: "Enable", reveals: [{ input: "Secret" }] },
      { input: "Dependent", when: "secret" },
      { text: "Inverse", when: "!dependent" },
      { select: "Mode", options: ["A", "B"], A: [{ check: "Nested" }] },
      { text: "Nested info", when: "nested" },
    ]);
    const evaluator = createVisibilityEvaluator(f);
    let answers = { ...f.defaults };
    const same = () =>
      expect([...evaluator.update(answers)].sort()).toEqual(
        [...computeVisibility(f, answers)].sort(),
      );
    same();
    for (const [id, value] of [
      ["enable", true],
      ["secret", "hi"],
      ["dependent", "answer"],
      ["enable", false],
      ["enable", true],
      ["mode", "A"],
      ["nested", true],
      ["mode", "B"],
      ["mode", null],
    ] as const) {
      answers = { ...answers, [id]: value };
      expect([...evaluator.update(answers, [id])].sort()).toEqual(
        [...computeVisibility(f, answers)].sort(),
      );
    }
    const before = evaluator.update(answers);
    answers = { ...answers, secret: "changed" };
    expect(evaluator.update(answers, ["secret"])).toBe(before);
  });

  it("rejects empty aggregate functions and malformed calls and escapes", () => {
    for (const when of [
      "any()",
      "all()",
      "count()",
      "count(true)",
      "count(a,b)",
      "selected(a)",
      'selected(a, "unterminated)',
      "unknown(a)",
      'a == "\\q"',
      'a == "\\uZZZZ"',
    ]) {
      expect(() =>
        form([{ check: "A" }, { text: "Conditional", when }]),
      ).toThrow(DefinitionError);
    }
  });
  it("accepts legacy references, quoted selected literals and decoded escapes", () => {
    const f = form([
      {
        select: "Choice",
        options: ["it's fine", 'double "quote"', "Unicode A"],
        default: "it's fine",
      },
      { input: "Note", default: "line\nnext" },
      {
        text: "Alias",
        id: "alias",
        when: "selected(@choice, 'it\\'s fine') && @note == 'line\\nnext'",
      },
      {
        text: "Unicode",
        id: "unicode",
        when: 'selected(choice, "Unicode \\u0041")',
      },
    ]);
    expect(ids(f, f.defaults)).toContain("alias");
    expect(ids(f, { ...f.defaults, choice: "Unicode A" })).toContain("unicode");
  });
  it("compares strings lexically and scalar types without coercion", () => {
    const f = form([
      { input: "Text", default: "10" },
      { slider: "Number", min: 0, max: 20, default: 10 },
      { text: "Equal", id: "equal", when: "text == number" },
      { text: "Lexical", id: "lexical", when: 'text < "2"' },
      { text: "Different", id: "different", when: "text != number" },
      { text: "Mixed", id: "mixed", when: "text >= number" },
    ]);
    const shown = ids(f, f.defaults);
    expect(shown).not.toContain("equal");
    expect(shown).toContain("lexical");
    expect(shown).toContain("different");
    expect(shown).not.toContain("mixed");
  });
  it("injects Other text for author supplied aliases without branches", () => {
    for (const other of ["Other", "other", "OTHER (PLEASE SPECIFY)"]) {
      const f = form([
        {
          page: "Page",
          elements: [{ multi: "Pick", options: ["A", other], default: other }],
        },
      ]);
      expect(f.byId.get("pick")?.options).toHaveLength(2);
      expect(
        serializeResult(f, { ...f.defaults, pick_other_text: "custom" }, ""),
      ).toMatchObject({ pick: [other], pick_other_text: "custom" });
    }
  });
  it("rejects prototype keys, non-JSON objects and sparse arrays", () => {
    for (const id of ["__proto__", "constructor", "prototype"])
      expect(() => form([{ input: "Unsafe", id }])).toThrow(/reserved/);
    expect(() =>
      compileDefinition(Object.create({ title: "Inherited", elements: [] })),
    ).toThrow(/plain JSON/);
    expect(() => form(Array(1))).toThrow(DefinitionError);
    expect(() => form([{ select: "Sparse", options: Array(1) }])).toThrow(
      DefinitionError,
    );
    const f = form([{ multi: "M", options: ["A"] }]);
    expect(() => serializeResult(f, { m: Array(1) }, "")).toThrow(
      /unique option values/,
    );
    const payload = JSON.parse('{"__proto__":true}');
    expect(() => serializeResult(f, payload, "")).toThrow(/Unknown answer/);
    expect({}).not.toHaveProperty("polluted");
  });
  it("bounds UTF-8 payload size and safely handles the longest dependency chain", () => {
    expect(() => form([{ text: "😀".repeat(260000) }])).toThrow(/1 MB/);
    const elements = Array.from({ length: 1000 }, (_, i) => ({
      check: `Node ${i}`,
      id: `n${i}`,
      default: true,
      ...(i < 999 ? { when: `n${i + 1}` } : {}),
    }));
    const f = form(elements);
    expect(computeVisibility(f, f.defaults).size).toBe(1000);
    elements[999] = { ...elements[999], when: "n0" };
    expect(() => form(elements)).toThrow(/cycle/);
  });
  it("reevaluates only the affected closure on a large unrelated form", () => {
    const f = form([
      { input: "Trigger" },
      { text: "Dependent", when: "trigger" },
      ...Array.from({ length: 900 }, (_, i) => ({
        check: `Unrelated ${i}`,
        when: "true",
      })),
    ]);
    const evaluator = createVisibilityEvaluator(f);
    let answers = { ...f.defaults };
    const first = evaluator.update(answers);
    let reads = 0;
    for (const node of f.all.slice(2)) {
      const condition = node.when;
      Object.defineProperty(node, "when", {
        get() {
          reads++;
          return condition;
        },
      });
    }
    answers = { ...answers, trigger: "yes" };
    const second = evaluator.update(answers, ["trigger"]);
    expect(second).not.toBe(first);
    expect(reads).toBe(0);
    answers = { ...answers, trigger: "more" };
    expect(evaluator.update(answers, ["trigger"])).toBe(second);
    expect(reads).toBe(0);
    expect([...second].sort()).toEqual(
      [...computeVisibility(f, answers)].sort(),
    );
  });
  it("snaps implicit slider defaults while preserving authored off-grid values", () => {
    const f = form([
      { slider: "Implicit", min: 0, max: 3 },
      { slider: "Authored", min: 1000, max: 10000, step: 100, default: 8080 },
      { slider: "Decimal", min: 0.1, max: 0.6, step: 0.1 },
      { slider: "Endpoint", min: 0, max: 10, step: 3 },
    ]);
    expect(f.defaults.implicit).toBe(2);
    expect(f.defaults.decimal).toBe(0.4);
    expect(f.defaults.authored).toBe(8080);
    const decimal = f.byId.get("decimal")!,
      endpoint = f.byId.get("endpoint")!;
    expect(snapSliderValue(decimal, 0.30000000000000004)).toBe(0.3);
    expect(snapSliderValue(endpoint, 10)).toBe(10);
    expect(snapSliderValue(endpoint, 9.6)).toBe(10);
    expect(snapSliderValue(endpoint, 7.4)).toBe(6);
    expect(snapSliderValue(endpoint, -10)).toBe(0);
    expect(snapSliderValue(endpoint, 20)).toBe(10);
    expect(serializeResult(f, f.defaults, "")).toHaveProperty("authored", 8080);
  });
  it("retains author Other children and guarantees an unconditional canonical text input", () => {
    for (const children of [
      [],
      [{ text: "Context" }],
      [{ input: "Custom reason", id: "reason" }],
    ]) {
      const f = form([
        { select: "Pick", options: ["A", "Other"], Other: children },
      ]);
      const branch = f.byId.get("pick")!.branches.Other;
      expect(branch).toHaveLength(children.length + 1);
      expect(
        serializeResult(
          f,
          { ...f.defaults, pick: "Other", pick_other_text: "Unlisted choice" },
          "",
        ),
      ).toHaveProperty("pick_other_text", "Unlisted choice");
      expect(new Set(f.all.map((n) => n.key)).size).toBe(f.all.length);
    }
    const f = form([
      {
        select: "Pick",
        options: ["Other"],
        Other: [{ input: "Custom label", id: "pick_other_text" }],
      },
    ]);
    expect(f.byId.get("pick")!.branches.Other).toHaveLength(1);
  });
});

describe("stable option identities and disclosures", () => {
  it("keeps stable defaults, gates and results unchanged when labels change", () => {
    const make = (label: string) =>
      form([
        {
          select: "Strategy",
          id: "strategy",
          options: [
            { id: "repair", label, children: [{ input: "Plan", id: "plan" }] },
            { id: "replace", label: "Replace" },
          ],
          default: "repair",
        },
        {
          text: "Repair selected",
          id: "status_text",
          when: "selected(strategy, repair)",
        },
      ]);
    for (const label of ["Repair", "Keep and improve the existing system"]) {
      const f = make(label);
      expect(f.defaults.strategy).toBe("repair");
      expect(ids(f, f.defaults)).toContain("plan");
      expect(ids(f, f.defaults)).toContain("status_text");
      expect(
        serializeResult(f, { ...f.defaults, plan: "Add coverage" }, ""),
      ).toMatchObject({
        strategy: "repair",
        plan: "Add coverage",
        _option_labels: { strategy: { repair: label } },
      });
      expect(
        serializeResult(f, { ...f.defaults, strategy: "replace" }, ""),
      ).not.toHaveProperty("plan");
    }
  });
  it("returns selected labels only for active stable widgets including rank and mixed options", () => {
    const f = form([
      {
        check: "Reveal",
        id: "reveal",
        reveals: [
          {
            select: "Hidden",
            options: [{ id: "hidden", label: "Hidden label" }],
            default: "hidden",
          },
        ],
      },
      {
        multi: "Features",
        options: [{ id: "logs", label: "Detailed logs" }, "Legacy"],
        default: ["logs", "Legacy"],
      },
      {
        rank: "Priorities",
        options: [
          { id: "reliability", label: "Reliable" },
          { id: "speed", label: "Fast" },
        ],
      },
      { select: "Empty", options: [{ id: "empty", label: "Empty label" }] },
      { select: "Legacy only", options: ["A"], default: "A" },
    ]);
    const result = serializeResult(
      f,
      { ...f.defaults, priorities: ["speed", "reliability"] },
      "",
    );
    expect(result).toMatchObject({
      priorities: ["speed", "reliability"],
      features: ["logs", "Legacy"],
      _option_labels: {
        features: { logs: "Detailed logs", Legacy: "Legacy" },
        priorities: { speed: "Fast", reliability: "Reliable" },
      },
    });
    expect(
      Object.keys(
        (result as Record<string, unknown>)._option_labels as object,
      ).sort(),
    ).toEqual(["features", "priorities"]);
    expect(result).not.toHaveProperty("hidden");
    expect(
      serializeResult(
        form([{ select: "Legacy", options: ["A"], default: "A" }]),
        { legacy: "A" },
        "",
      ),
    ).not.toHaveProperty("_option_labels");
  });
  it("treats recommendations as presentation rather than answer defaults", () => {
    const f = form([
      {
        select: "Choice",
        options: [
          {
            id: "recommended",
            label: "Repair",
            recommendation: "Smallest change",
            because: "Preserves the boundary",
          },
          { id: "alternative", label: "Replace" },
        ],
      },
      {
        multi: "Legacy",
        options: [{ value: "A", recommendation: "Good starting point" }],
      },
    ]);
    expect(f.defaults.choice).toBeNull();
    expect(f.defaults.legacy).toEqual([]);
    expect(f.byId.get("choice")!.options[0]).toMatchObject({
      recommendation: "Smallest change",
      description: "Preserves the boundary",
    });
    for (const recommendation of ["", "  ", true])
      expect(() =>
        form([
          {
            select: "Bad",
            options: [{ id: "bad", label: "Bad", recommendation }],
          },
        ]),
      ).toThrow(/Recommendation|Expected a string/);
  });
  it("allows property-named stable IDs with inline children and retains unambiguous external branches", () => {
    const f = form([
      {
        select: "Mode",
        options: [
          {
            id: "default",
            label: "Use defaults",
            children: [{ input: "Confirm" }],
          },
          { id: "custom", label: "Customize" },
        ],
        default: "default",
        custom: [{ input: "Custom settings" }],
      },
    ]);
    expect(ids(f, f.defaults)).toContain("confirm");
    expect(ids(f, { ...f.defaults, mode: "custom" })).toContain(
      "custom_settings",
    );
    expect(() => form([{ select: "Legacy", options: ["default"] }])).toThrow(
      /collides/,
    );
  });
  it("validates stable shapes, duplicates, children and catchall identity collisions", () => {
    for (const option of [
      { id: "a", label: "A", value: "A" },
      { id: "a" },
      { label: "A" },
      { id: "bad-id", label: "A" },
      { id: "a", label: " " },
      { value: "A", children: [] },
      { id: "_other", label: "Not a catchall" },
    ])
      expect(() => form([{ select: "Bad", options: [option] }])).toThrow(
        DefinitionError,
      );
    expect(() =>
      form([
        {
          select: "Bad",
          options: [{ id: "same", label: "One" }, { value: "same" }],
        },
      ]),
    ).toThrow(/unique/);
    expect(() =>
      form([
        {
          select: "Bad",
          options: [{ id: "a", label: "A", children: [] }],
          a: [],
        },
      ]),
    ).toThrow(/not both/);
    expect(() =>
      form([
        {
          rank: "Bad",
          options: [
            { id: "a", label: "A", children: [] },
            { id: "b", label: "B" },
          ],
        },
      ]),
    ).toThrow(/Rank options cannot/);
    expect(() =>
      form([{ select: "Bad", options: [{ id: "a", label: "A" }, "_other"] }]),
    ).toThrow(/collides/);
    expect(() => form([{ input: "Reserved", id: "_option_labels" }])).toThrow(
      /reserved/,
    );
    expect(() =>
      form([
        {
          select: "Bad",
          options: [{ id: "a", label: "A", children: { input: "Not array" } }],
        },
      ]),
    ).toThrow(/array/);
  });
  it("injects stable Other with labels and preserves explicit catchall children", () => {
    for (const explicit of [false, true]) {
      const f = form([
        {
          multi: "Choose",
          options: [
            { id: "a", label: "A" },
            ...(explicit
              ? [
                  {
                    id: "_other",
                    label: "Other",
                    children: [{ text: "Extra context" }],
                  },
                ]
              : []),
          ],
        },
      ]);
      expect(
        f.byId.get("choose")!.options.find((o) => o.value === "_other"),
      ).toMatchObject({
        stable: true,
        label: explicit ? "Other" : "Other (please specify)",
      });
      expect(
        serializeResult(
          f,
          {
            ...f.defaults,
            choose: ["_other"],
            choose_other_text: "My approach",
          },
          "",
        ),
      ).toMatchObject({
        choose: ["_other"],
        choose_other_text: "My approach",
        _option_labels: {
          choose: { _other: explicit ? "Other" : "Other (please specify)" },
        },
      });
    }
    const f = form([
      { select: "Explicit", options: [{ id: "custom_other", label: "Other" }] },
    ]);
    expect(f.byId.get("explicit")!.options).toHaveLength(1);
    expect(f.byId.has("explicit_other_text")).toBe(true);
  });
  it("disclosure open affects presentation only and when still controls descendants", () => {
    for (const open of [false, true]) {
      const f = form([
        { check: "Enable", default: true },
        {
          disclosure: "Inspect proposal",
          id: "proposal",
          open,
          when: "enable",
          elements: [{ input: "Draft", default: "Keep this proposal" }],
        },
      ]);
      expect(f.byId.get("proposal")!.open).toBe(open);
      expect(ids(f, f.defaults)).toContain("draft");
      expect(serializeResult(f, f.defaults, "")).toHaveProperty(
        "draft",
        "Keep this proposal",
      );
      expect(serializeResult(f, f.defaults, "")).not.toHaveProperty("proposal");
      expect(
        serializeResult(f, { ...f.defaults, enable: false }, ""),
      ).not.toHaveProperty("draft");
    }
    const f = form([{ disclosure: "Default collapsed", elements: [] }]);
    expect(f.nodes[0].open).toBe(false);
    expect(() =>
      form([{ disclosure: "Invalid", elements: [], open: "yes" }]),
    ).toThrow(/boolean/);
    expect(() =>
      form([
        { disclosure: "Cycle", when: "child", elements: [{ check: "Child" }] },
      ]),
    ).toThrow(/cycle/);
  });
});

describe("defaults and the readable summary", () => {
  const f = compileDefinition({
    elements: [
      { check: "Keep tree", id: "keep", default: true },
      { slider: "Runs", id: "runs", min: 1, max: 9, default: 3 },
      { select: "Step", id: "step", options: ["a", "b"] },
      { input: "Command", id: "cmd" },
    ],
  });
  it("lists the fields the user never changed", () => {
    const r = serializeResult(
      f,
      { ...f.defaults, runs: 5 },
      "",
      undefined,
      new Set(["runs"]),
    );
    expect(r).toMatchObject({ _defaulted: ["keep", "step", "cmd"] });
  });
  it("leaves unanswered fields out of the summary and marks defaults", () => {
    const r = serializeResult(
      f,
      { ...f.defaults, runs: 5 },
      "",
      undefined,
      new Set(["runs"]),
    );
    expect(
      summaryRows(r).map((row) => [row.label, row.value, row.defaulted]),
    ).toEqual([
      ["Keep tree", "Yes", true],
      ["Runs", "5", false],
    ]);
  });
});
