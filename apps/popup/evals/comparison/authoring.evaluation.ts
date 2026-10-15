import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { fromJsonSchema } from "@modelcontextprotocol/server";
import {
  compileDefinition,
  computeVisibility,
  serializeResult,
} from "../../src/core";
import type { Answers } from "../../src/core";

const directory = "evals/comparison";
const read = (path: string) => JSON.parse(readFileSync(path, "utf8"));
const write = (path: string, data: unknown) =>
  writeFileSync(path, JSON.stringify(data, null, 2) + "\n");
const tasks = read(`${directory}/tasks.json`) as {
  id: string;
  request: string;
  context: string;
  combined_intent: string;
  correction: string;
}[];
const baseline = read("evals/baseline-authoring.json");
const candidate = read("evals/candidate-authoring.json");
baseline.inputSchema.required = ["elements"];
const schema = fromJsonSchema(candidate.inputSchema);
const semantic = (value: unknown): unknown => {
  if (Array.isArray(value)) return value.map(semantic);
  if (value && typeof value === "object")
    return Object.fromEntries(
      Object.entries(value)
        .filter(([key]) => !["description", "examples"].includes(key))
        .map(([key, item]) => [key, semantic(item)]),
    );
  return value;
};

describe("recorded independent authoring comparison", () => {
  it("compares equivalent DSLs and records validity and attention proxies", async () => {
    expect(semantic(baseline.inputSchema)).toEqual(
      semantic(candidate.inputSchema),
    );
    mkdirSync(`${directory}/blind`, { recursive: true });
    const files = readdirSync(`${directory}/candidates`)
      .filter((file) =>
        /^[bc]-(ci|performance|exploration|evidence)-[12]\.json$/.test(file),
      )
      .sort();
    expect(files).toHaveLength(16);
    const records: Record<string, unknown>[] = [];
    const blind = new Map<string, Record<string, unknown>[]>();
    for (const file of files) {
      const definition = read(`${directory}/candidates/${file}`);
      const task = tasks.find((task) => file.includes(`-${task.id}-`))!;
      const id = createHash("sha256")
        .update(`popup-conversation-v1:${file}`)
        .digest("hex")
        .slice(0, 8);
      const validation = await schema["~standard"].validate(definition);
      let record: Record<string, unknown> = {
        file,
        id,
        task: task.id,
        variant: file[0],
        schemaValid: !("issues" in validation),
        definitionBytes: Buffer.byteLength(JSON.stringify(definition)),
      };
      let fields: unknown = [];
      let defaults: Answers = {};
      try {
        const form = compileDefinition(definition);
        const visible = computeVisibility(form, form.defaults);
        defaults = form.defaults;
        fields = form.all
          .filter((node) => node.id)
          .map((node) => ({
            id: node.id,
            kind: node.kind,
            options: node.options,
            parent: node.parent,
            gate: node.gate,
          }));
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
        record = {
          ...record,
          compilerValid: true,
          initialWords: words(true),
          allWords: words(false),
          initialControls: form.all.filter(
            (node) => node.id && visible.has(node.key),
          ).length,
          allControls: form.byId.size,
        };
      } catch (error) {
        record = { ...record, compilerValid: false, error: String(error) };
      }
      records.push(record);
      const group = blind.get(task.id) ?? [];
      group.push({ id, definition, fields, defaults });
      blind.set(task.id, group);
    }
    write(`${directory}/validation.json`, records);
    for (const task of tasks)
      write(`${directory}/blind/${task.id}.json`, {
        task,
        candidates: blind
          .get(task.id)!
          .sort((a, b) => String(a.id).localeCompare(String(b.id))),
      });
    // Invalid first attempts are recorded, never silently repaired or discarded.
    console.log(
      JSON.stringify(
        records.map(({ file, compilerValid, initialWords }) => ({
          file,
          compilerValid,
          initialWords,
        })),
      ),
    );
  });

  it("replays graded candidate-specific selections and corrections through the actual serializer", () => {
    mkdirSync(`${directory}/replays`, { recursive: true });
    for (const task of tasks) {
      let grades: {
        candidates: {
          id: string;
          combined_answers: Answers;
          other_answers: Answers;
        }[];
      };
      grades = read(`${directory}/grades-${task.id}.json`);
      expect(grades.candidates).toHaveLength(4);
      const records = read(`${directory}/validation.json`) as {
        id: string;
        file: string;
        compilerValid: boolean;
      }[];
      for (const grade of grades.candidates) {
        const record = records.find((record) => record.id === grade.id)!;
        if (!record.compilerValid) continue;
        const form = compileDefinition(
          read(`${directory}/candidates/${record.file}`),
        );
        const pages = new Set(
          form.all
            .filter((node) => node.kind === "page")
            .map((node) => node.key),
        );
        const replay = (overrides: Answers, notes = "") => {
          for (const id of Object.keys(overrides))
            expect(
              form.byId.has(id),
              `Unknown replay id ${id} in ${record.file}`,
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
          write(`${directory}/replays/${grade.id}-${item.name}.json`, {
            type: "popup_result",
            formId: `eval-${grade.id}`,
            submissionId: `eval-${grade.id}-${item.name}`,
            result: item.result,
          });
      }
    }
  });
});
