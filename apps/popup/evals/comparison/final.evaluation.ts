import { readFileSync, readdirSync, writeFileSync } from "node:fs";
import { expect, it } from "vitest";
import { fromJsonSchema } from "@modelcontextprotocol/server";
import {
  compileDefinition,
  computeVisibility,
  serializeResult,
} from "../../src/core";
import type { Answers } from "../../src/core";

it("records follow-up samples against their frozen October 9 authoring bundle", async () => {
  const files = readdirSync("evals/comparison/candidates").filter(
    (file) => file.startsWith("f-") && file.endsWith(".json"),
  );
  expect(files).toHaveLength(4);
  const bundle = JSON.parse(readFileSync("evals/final-authoring.json", "utf8"));
  const schema = fromJsonSchema(
    bundle.inputSchema as Parameters<typeof fromJsonSchema>[0],
  );
  const records = [];
  for (const file of files.sort()) {
    const definition = JSON.parse(
      readFileSync(`evals/comparison/candidates/${file}`, "utf8"),
    );
    expect(await schema["~standard"].validate(definition)).not.toHaveProperty(
      "issues",
    );
    const form = compileDefinition(definition);
    const visible = computeVisibility(form, form.defaults);
    const words = (initial: boolean) =>
      form.all
        .filter((node) => !initial || visible.has(node.key))
        .map((node) =>
          [
            node.label,
            ...node.options.map(
              (option) => `${option.value} ${option.description ?? ""}`,
            ),
            node.id && ["input", "slider"].includes(node.kind)
              ? String(form.defaults[node.id] ?? "")
              : "",
          ].join(" "),
        )
        .join(" ")
        .trim()
        .split(/\s+/).length;
    records.push({
      file,
      compilerValid: true,
      schemaValid: true,
      initialWords: words(true),
      allWords: words(false),
      initialControls: form.all.filter(
        (node) => node.id && visible.has(node.key),
      ).length,
      allControls: form.byId.size,
    });
  }
  writeFileSync(
    "evals/comparison/final-validation.json",
    JSON.stringify(records, null, 2) + "\n",
  );
});

it("replays the final samples with their actual fields", () => {
  let grades: {
    candidates: {
      file: string;
      combined_answers: Answers;
      other_answers: Answers;
    }[];
  };
  grades = JSON.parse(
    readFileSync("evals/comparison/grades-final.json", "utf8"),
  );
  const tasks = JSON.parse(
    readFileSync("evals/comparison/tasks.json", "utf8"),
  ) as { id: string; correction: string; combined_intent: string }[];
  expect(grades.candidates).toHaveLength(4);
  for (const grade of grades.candidates) {
    const form = compileDefinition(
      JSON.parse(
        readFileSync(`evals/comparison/candidates/${grade.file}`, "utf8"),
      ),
    );
    const task = tasks.find((task) => grade.file === `f-${task.id}.json`)!;
    const pages = new Set(
      form.all.filter((node) => node.kind === "page").map((node) => node.key),
    );
    const replay = (overrides: Answers, notes = "") => {
      for (const id of Object.keys(overrides))
        expect(
          form.byId.has(id),
          `Unknown replay id ${id} in ${grade.file}`,
        ).toBe(true);
      const answers = { ...form.defaults, ...overrides };
      const result = serializeResult(form, answers, notes, pages);
      const visible = computeVisibility(form, answers);
      for (const node of form.all)
        if (node.id && !visible.has(node.key))
          expect(result).not.toHaveProperty(node.id);
      return result;
    };
    const cases = [
      { name: "combined", result: replay(grade.combined_answers) },
      { name: "other", result: replay(grade.other_answers) },
      {
        name: "contradiction",
        result: replay(grade.combined_answers, task.correction),
      },
      { name: "cancel", result: { status: "cancelled" } },
    ];
    expect(cases[2].result).toHaveProperty("_notes", task.correction);
    expect(
      Object.entries(cases[1].result).some(
        ([key, value]) =>
          key.endsWith("_other_text") && value === task.combined_intent,
      ),
    ).toBe(true);
    for (const item of cases)
      writeFileSync(
        `evals/comparison/replays/${grade.file.slice(0, -5)}-${item.name}.json`,
        JSON.stringify(
          {
            type: "popup_result",
            formId: `eval-${grade.file}`,
            submissionId: `eval-${grade.file}-${item.name}`,
            result: item.result,
          },
          null,
          2,
        ) + "\n",
      );
  }
});
