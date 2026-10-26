import { describe, expect, it } from "vitest";
import { compileDefinition, computeVisibility, serializeResult } from "../core";
import { fromJsonSchema } from "@modelcontextprotocol/server";
import { ciWorkedExample, popupDescription, popupInputSchema } from "./index";
import { densityPairs } from "../../evals/density";
import { repairCases, scenarios } from "../../evals/scenarios";

describe("model-facing authoring assets", () => {
  it("keeps every published example and reference fixture executable", async () => {
    const schema = fromJsonSchema(
      popupInputSchema as Parameters<typeof fromJsonSchema>[0],
    );
    for (const definition of [
      ...popupInputSchema.examples,
      ...scenarios.map((s) => s.definition),
      ...densityPairs.flatMap((pair) => [pair.overlong, pair.concise]),
    ]) {
      expect(() => compileDefinition(definition)).not.toThrow();
      expect(await schema["~standard"].validate(definition)).not.toHaveProperty(
        "issues",
      );
    }
  });
  it("supports stable simultaneous explorations and a shared contribution", () => {
    const form = compileDefinition(ciWorkedExample);
    const answers = {
      ...form.defaults,
      changes: ["cache", "parallel"],
      db_isolation: "investigate",
    };
    const shown = form.all.filter((n) =>
      computeVisibility(form, answers).has(n.key),
    );
    expect(shown.map((n) => n.id)).toContain("cache_summary");
    expect(shown.map((n) => n.id)).toContain("next_goal");
    expect(shown.map((n) => n.id)).not.toContain("workers");
    expect(
      shown.some((n) => n.label.includes("separately before combining")),
    ).toBe(true);
    const result = serializeResult(
      form,
      answers,
      "Explore first; no changes yet.",
    );
    expect(result).not.toHaveProperty("isolation_design");
    expect(result).not.toHaveProperty("coverage");
    expect(result).toHaveProperty("_option_labels.changes", {
      cache: "Cache dependencies",
      parallel: "Parallelize integration tests",
    });
    // The editable proposal stays in the active branch, outside supplementary detail.
    expect(result).toHaveProperty("cache_summary");
  });
  it("keeps recommended exploratory branches initially empty", () => {
    const form = compileDefinition(ciWorkedExample);
    const shownIds = form.all
      .filter((n) => computeVisibility(form, form.defaults).has(n.key))
      .map((n) => n.id);
    expect(form.defaults.changes).toEqual([]);
    expect(shownIds).not.toContain("cache_miss");
    expect(shownIds).not.toContain("db_isolation");
    expect(shownIds).not.toContain("next_goal");
    expect(form.byId.get("changes")?.options[0].recommendation).toBeTruthy();
  });
  it("advertises disclosure and stable options while keeping rank children invalid", async () => {
    const schema = fromJsonSchema(
      popupInputSchema as Parameters<typeof fromJsonSchema>[0],
    );
    const invalid = {
      elements: [
        {
          rank: "Order",
          options: [
            { id: "a", label: "A", children: [] },
            { id: "b", label: "B" },
          ],
        },
      ],
    };
    expect(await schema["~standard"].validate(invalid)).toHaveProperty(
      "issues",
    );
    expect(() => compileDefinition(invalid)).toThrow();
  });
  it("removes repetition in fixed pairs without losing reasoning or changing answers", () => {
    for (const pair of densityPairs) {
      const concise = compileDefinition(pair.concise);
      const overlong = compileDefinition(pair.overlong);
      const answers = {
        ...concise.defaults,
        directions: ["a", "b"],
        next: "Explore consequences",
      };
      const visibleText = (form: ReturnType<typeof compileDefinition>) =>
        form.all
          .filter((node) => computeVisibility(form, answers).has(node.key))
          .map((node) =>
            [
              node.label,
              ...node.options.map((option) => option.description ?? ""),
            ].join(" "),
          )
          .join(" ");
      for (const claim of pair.substantiveClaims)
        expect(visibleText(concise)).toContain(claim);
      expect(visibleText(concise).length).toBeLessThan(
        visibleText(overlong).length,
      );
      expect(serializeResult(concise, answers, "Correction")).toEqual(
        serializeResult(overlong, answers, "Correction"),
      );
      expect(concise.defaults.directions).toEqual([]);
      expect(concise.all.filter((node) => node.id === "next")).toHaveLength(1);
    }
  });
  it("keeps fixed repair cases invalid until corrected", () => {
    for (const example of repairCases)
      expect(() => compileDefinition(example.definition)).toThrow();
  });
  it("preserves recursive oneOf metadata and the asynchronous result contract", () => {
    expect(popupInputSchema.properties.elements.items.oneOf).toHaveLength(11);
    expect(JSON.stringify(popupInputSchema)).toContain("#/properties/elements");
    expect(popupDescription).toContain("awaiting_input");
    expect(popupDescription).toContain("popup_result");
    expect(popupInputSchema.required).toEqual(["elements"]);
    expect(popupInputSchema.examples).toEqual([ciWorkedExample]);
    const page = popupInputSchema.properties.elements.items.oneOf.find(
      (e) => "page" in e.properties,
    )!;
    expect(page.properties).not.toHaveProperty("next");
  });
});
