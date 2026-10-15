import { evaluate, parseCondition, truthy } from "./condition";
import {
  DefinitionError,
  type Answers,
  type CompiledForm,
  type Node,
  type PopupResult,
  type Expr,
} from "./types";
export * from "./types";
const kinds = [
  "text",
  "markdown",
  "slider",
  "check",
  "input",
  "select",
  "multi",
  "rank",
  "group",
  "disclosure",
  "page",
] as const;
const widgets = new Set([
  "slider",
  "check",
  "input",
  "select",
  "multi",
  "rank",
]);
const reserved = new Set([
  "status",
  "button",
  "error",
  "_notes",
  "_labels",
  "_option_labels",
  "_route",
  "__proto__",
  "constructor",
  "prototype",
]);
function fail(path: string, message: string): never {
  throw new DefinitionError([{ path, message }]);
}
const object = (v: unknown, p: string): Record<string, unknown> => {
  if (
    !v ||
    typeof v !== "object" ||
    Array.isArray(v) ||
    ![Object.prototype, null].includes(Object.getPrototypeOf(v))
  )
    fail(p, "Expected a plain JSON object");
  return v as Record<string, unknown>;
};
const string = (v: unknown, p: string): string => {
  if (typeof v !== "string" || v.length > 100000)
    fail(p, "Expected a string of at most 100000 characters");
  return v as string;
};
const number = (v: unknown, p: string): number => {
  if (typeof v !== "number" || !Number.isFinite(v))
    fail(p, "Expected a finite number");
  return v as number;
};
export function labelToId(label: string): string {
  let result = "",
    separator = true,
    upper = false;
  const chars = [...label];
  for (let i = 0; i < chars.length; i++) {
    const c = chars[i],
      isUpper = /\p{Lu}/u.test(c);
    if (/[\p{L}\p{N}]/u.test(c)) {
      if (
        isUpper &&
        result &&
        !separator &&
        (!upper || /\p{Ll}/u.test(chars[i + 1] ?? ""))
      )
        result += "_";
      result += /[A-Z]/.test(c) ? c.toLowerCase() : c;
      separator = false;
      upper = isUpper;
    } else if ([" ", "-", "_"].includes(c)) {
      if (result && !separator) result += "_";
      separator = true;
      upper = false;
    } else upper = false;
  }
  return result.replace(/_+$/, "");
}
function suggestion(value: string, candidates: Iterable<string>): string {
  if (value.length > 128) return "";
  let best = "",
    distance = Infinity;
  for (const candidate of candidates) {
    if (candidate.length > 128) continue;
    let row = Array.from({ length: candidate.length + 1 }, (_, i) => i);
    for (let i = 0; i < value.length; i++) {
      const next = [i + 1];
      for (let j = 0; j < candidate.length; j++)
        next.push(
          Math.min(
            next[j] + 1,
            row[j + 1] + 1,
            row[j] + (value[i] === candidate[j] ? 0 : 1),
          ),
        );
      row = next;
    }
    if (row[candidate.length] < distance) {
      distance = row[candidate.length];
      best = candidate;
    }
  }
  return best && distance <= Math.max(2, Math.floor(value.length / 3))
    ? `; did you mean "${best}"?`
    : "";
}
export function compileDefinition(input: unknown): CompiledForm {
  const raw = object(input, "$");
  let bytes: number;
  try {
    bytes = new TextEncoder().encode(JSON.stringify(input)).length;
  } catch {
    return fail("$", "Definition must be serializable JSON");
  }
  if (bytes > 1000000) fail("$", "Definition exceeds 1 MB");
  for (const k of Object.keys(raw))
    if (!["title", "elements", "submit", "notes"].includes(k))
      fail(
        `$.${k}`,
        "Unknown field; only classic forms with elements are supported",
      );
  const form: CompiledForm = {
    title: raw.title === undefined ? "" : string(raw.title, "$.title"),
    submit:
      raw.submit === undefined ? "Submit" : string(raw.submit, "$.submit"),
    notes: raw.notes === undefined ? true : (raw.notes as boolean),
    nodes: [],
    defaults: Object.create(null),
    all: [],
    byId: new Map(),
    byKey: new Map(),
    topo: [],
  };
  if (typeof form.notes !== "boolean") fail("$.notes", "Expected a boolean");
  const list = (
    value: unknown,
    path: string,
    parent?: Node,
    gate?: Node["gate"],
    page?: string,
    depth = 0,
  ): Node[] => {
    if (!Array.isArray(value)) fail(path, "Expected an array of elements");
    if (depth > 32) fail(path, "Nesting exceeds 32 levels");
    return Array.from(value as unknown[]).map((v, i) => {
      const p = `${path}[${i}]`,
        r = object(v, p);
      const tags = kinds.filter((k) => Object.hasOwn(r, k));
      if (tags.length !== 1) fail(p, "Expected exactly one element kind");
      const kind = tags[0];
      if (kind === "page" && parent)
        fail(p, "Pages are allowed only at the top level");
      const label = string(r[kind], `${p}.${kind}`);
      const node: Node = {
        key: p,
        kind,
        label,
        options: [],
        elements: [],
        reveals: [],
        branches: Object.create(null),
        refs: [],
        parent: parent?.key,
        gate,
        page,
      };
      if (kind === "page") node.page = node.key;
      if (r.id !== undefined || widgets.has(kind)) {
        node.id =
          r.id === undefined ? labelToId(label) : string(r.id, `${p}.id`);
        if (!/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(node.id))
          fail(
            `${p}.id`,
            "Use a nonempty ID starting with a letter or underscore, containing letters, digits or underscores",
          );
        if (reserved.has(node.id)) fail(`${p}.id`, "ID is reserved");
        if (form.byId.has(node.id))
          fail(`${p}.id`, `Duplicate ID ${node.id}; use explicit unique IDs`);
        form.byId.set(node.id, node);
      }
      if (++count > 1000) fail(p, "Definition exceeds 1000 elements");
      form.all.push(node);
      form.byKey.set(node.key, node);
      const allowed = new Set([kind, "id", "when"]);
      if (kind === "slider")
        ["min", "max", "step", "default"].forEach((k) => allowed.add(k));
      if (kind === "check")
        ["default", "reveals"].forEach((k) => allowed.add(k));
      if (kind === "input")
        ["default", "rows", "placeholder"].forEach((k) => allowed.add(k));
      if (["select", "multi", "rank"].includes(kind)) allowed.add("options");
      if (["select", "multi"].includes(kind))
        ["default", "reveals"].forEach((k) => allowed.add(k));
      if (kind === "select") allowed.add("style");
      if (["group", "page", "disclosure"].includes(kind))
        allowed.add("elements");
      if (kind === "disclosure") {
        allowed.add("open");
        if (r.open !== undefined && typeof r.open !== "boolean")
          fail(`${p}.open`, "Expected a boolean");
        node.open = (r.open ?? false) as boolean;
      }
      if (r.when !== undefined) {
        try {
          const c = parseCondition(string(r.when, `${p}.when`));
          node.when = c.expr;
          node.refs = c.refs;
        } catch (e) {
          fail(`${p}.when`, (e as Error).message);
        }
      }
      if (["select", "multi", "rank"].includes(kind)) {
        if (
          !Array.isArray(r.options) ||
          !r.options.length ||
          r.options.length > 200
        )
          fail(`${p}.options`, "Expected 1–200 options");
        const inlineChildren = new Map<
          string,
          { value: unknown; path: string }
        >();
        node.options = Array.from(r.options as unknown[]).map((o, j) => {
          const op = `${p}.options[${j}]`;
          if (typeof o === "string") return { value: string(o, op) };
          const ob = object(o, op);
          const stable = Object.hasOwn(ob, "id") || Object.hasOwn(ob, "label");
          for (const k of Object.keys(ob))
            if (
              !(
                stable
                  ? [
                      "id",
                      "label",
                      "description",
                      "because",
                      "recommendation",
                      "children",
                    ]
                  : ["value", "description", "because", "recommendation"]
              ).includes(k)
            )
              fail(
                `${op}.${k}`,
                stable && k === "value"
                  ? "Use either id/label or value; do not mix option shapes"
                  : "Unknown option field",
              );
          if (ob.description !== undefined && ob.because !== undefined)
            fail(op, "Use either description or because");
          const value = string(
            stable ? ob.id : ob.value,
            `${op}.${stable ? "id" : "value"}`,
          );
          const display = stable ? string(ob.label, `${op}.label`) : value;
          if (!display.trim())
            fail(
              `${op}.${stable ? "label" : "value"}`,
              "Option label must be nonempty",
            );
          if (stable && !/^[a-zA-Z_][a-zA-Z0-9_]*$/.test(value))
            fail(
              `${op}.id`,
              "Option ID must start with a letter or underscore and contain only ASCII letters, digits or underscores",
            );
          if (
            stable &&
            value === "_other" &&
            !/^(other|other \(please specify\))$/i.test(display)
          )
            fail(`${op}.id`, "_other is reserved for an Other option");
          if (Object.hasOwn(ob, "children")) {
            if (kind === "rank")
              fail(`${op}.children`, "Rank options cannot have children");
            inlineChildren.set(value, {
              value: ob.children,
              path: `${op}.children`,
            });
          }
          const option: Node["options"][number] = {
            value,
            ...(stable ? { label: display, stable: true as const } : {}),
          };
          if (ob.description !== undefined || ob.because !== undefined)
            option.description = string(
              ob.description ?? ob.because,
              `${op}.description`,
            );
          if (ob.recommendation !== undefined) {
            option.recommendation = string(
              ob.recommendation,
              `${op}.recommendation`,
            );
            if (!option.recommendation.trim())
              fail(`${op}.recommendation`, "Recommendation must be nonempty");
          }
          return option;
        });
        const values = node.options.map((o) => o.value);
        if (values.some((v) => !v) || new Set(values).size !== values.length)
          fail(`${p}.options`, "Option values must be nonempty and unique");
        if (["select", "multi"].includes(kind)) {
          const propertyKeys = new Set(allowed);
          for (const normalized of node.options) {
            const option = normalized.value;
            if (
              (!normalized.stable && propertyKeys.has(option)) ||
              reserved.has(option)
            )
              fail(
                `${p}.options`,
                `Option ${option} collides with a reserved element key`,
              );
            allowed.add(option);
            const inline = inlineChildren.get(option);
            const external =
              !propertyKeys.has(option) && Object.hasOwn(r, option);
            if (inline && external)
              fail(
                `${p}.${option}`,
                "Use either inline children or an option-key branch for this option, not both",
              );
            if (inline || external)
              node.branches[option] = list(
                inline ? inline.value : r[option],
                inline ? inline.path : `${p}.${option}`,
                node,
                { id: node.id!, option },
                node.page,
                depth + 1,
              );
          }
          let other =
            node.options.find((o) => o.stable && o.value === "_other")?.value ??
            node.options.find((o) =>
              /^(other|other \(please specify\))$/i.test(o.label ?? o.value),
            )?.value;
          if (!other) {
            const stable = node.options.some((option) => option.stable);
            other = stable ? "_other" : "Other (please specify)";
            if (values.includes(other))
              fail(
                `${p}.options`,
                "Option identity collides with the automatic Other option",
              );
            node.options.push(
              stable
                ? {
                    value: other,
                    label: "Other (please specify)",
                    stable: true,
                  }
                : { value: other },
            );
          }
          const otherChildren = node.branches[other] ?? [];
          const otherId = `${node.id}_other_text`;
          // Author children remain alongside the guaranteed free-text escape hatch.
          if (
            !otherChildren.some(
              (child) =>
                child.kind === "input" && child.id === otherId && !child.when,
            )
          ) {
            otherChildren.push(
              ...list(
                [{ input: "Please specify", id: otherId }],
                `${p}.${other}.__injected`,
                node,
                { id: node.id!, option: other },
                node.page,
                depth + 1,
              ),
            );
          }
          node.branches[other] = otherChildren;
        }
      }
      for (const k of Object.keys(r))
        if (!allowed.has(k))
          fail(
            `${p}.${k}`,
            `Unknown field for ${kind}${kind === "page" && k === "next" ? "; pages use linear navigation" : suggestion(k, allowed)}`,
          );
      if (["group", "page", "disclosure"].includes(kind))
        node.elements = list(
          r.elements,
          `${p}.elements`,
          node,
          undefined,
          node.page,
          depth + 1,
        );
      if (["check", "select", "multi"].includes(kind))
        node.reveals = list(
          r.reveals ?? [],
          `${p}.reveals`,
          node,
          { id: node.id! },
          node.page,
          depth + 1,
        );
      if (kind === "slider") {
        node.min = number(r.min, `${p}.min`);
        node.max = number(r.max, `${p}.max`);
        node.step = r.step === undefined ? 1 : number(r.step, `${p}.step`);
        if (node.min >= node.max || node.step <= 0)
          fail(p, "Slider requires min < max and positive step");
        form.defaults[node.id!] =
          r.default === undefined
            ? snapSliderValue(node, node.min / 2 + node.max / 2)
            : number(r.default, `${p}.default`);
      }
      if (kind === "check") {
        if (r.default !== undefined && typeof r.default !== "boolean")
          fail(`${p}.default`, "Expected a boolean");
        form.defaults[node.id!] = (r.default ?? false) as boolean;
      }
      if (kind === "input") {
        form.defaults[node.id!] =
          r.default === undefined ? "" : string(r.default, `${p}.default`);
        if (r.placeholder !== undefined)
          node.placeholder = string(r.placeholder, `${p}.placeholder`);
        if (r.rows !== undefined) {
          node.rows = number(r.rows, `${p}.rows`);
          if (!Number.isInteger(node.rows) || node.rows < 1 || node.rows > 100)
            fail(`${p}.rows`, "Use an integer from 1 to 100");
        }
      }
      if (kind === "select") {
        form.defaults[node.id!] =
          r.default === undefined ? null : string(r.default, `${p}.default`);
        if (
          r.style !== undefined &&
          !["radio", "dropdown"].includes(r.style as string)
        )
          fail(`${p}.style`, "Use radio or dropdown");
        node.style = (r.style ??
          ((r.options as unknown[]).length > 6
            ? "dropdown"
            : "radio")) as Node["style"];
      }
      if (kind === "multi")
        form.defaults[node.id!] =
          r.default === undefined
            ? []
            : typeof r.default === "string"
              ? [r.default]
              : (r.default as string[]);
      if (kind === "rank") {
        if (node.options.length < 2)
          fail(`${p}.options`, "Rank requires at least two options");
        form.defaults[node.id!] = [];
      }
      return node;
    });
  };
  let count = 0;
  form.nodes = list(raw.elements, "$.elements");
  if (
    form.nodes.some((n) => n.kind === "page") &&
    form.nodes.some((n) => n.kind !== "page")
  )
    fail(
      "$.elements",
      "When using pages, every top-level element must be a page",
    );
  const states = new Map<string, number>();
  const visit = (node: Node): void => {
    if (states.get(node.key) === 1)
      fail(
        node.key,
        "Visibility dependency cycle; remove the condition or ancestor dependency",
      );
    if (states.get(node.key) === 2) return;
    states.set(node.key, 1);
    if (node.parent) visit(form.byKey.get(node.parent)!);
    for (const id of node.refs) {
      const dep = form.byId.get(id);
      if (!dep || !widgets.has(dep.kind))
        fail(
          `${node.key}.when`,
          `Unknown answer ID ${id}${suggestion(id, form.byId.keys())}`,
        );
      visit(dep);
    }
    states.set(node.key, 2);
    form.topo.push(node);
  };
  form.all.forEach(visit);
  const validateCondition = (expr: Expr, path: string): void => {
    if (expr.type === "call") {
      if (
        expr.name === "selected" &&
        expr.args[0].type === "ref" &&
        expr.args[1].type === "literal"
      ) {
        const target = form.byId.get(expr.args[0].id)!;
        const option = String(expr.args[1].value);
        const values =
          target.kind === "check"
            ? [target.id!]
            : target.options.map((o) => o.value);
        if (!values.includes(option))
          fail(
            path,
            `Unknown selected option "${option}" for ${target.id}${suggestion(option, values)}`,
          );
      }
      expr.args.forEach((e) => validateCondition(e, path));
    } else if (expr.type === "binary") {
      validateCondition(expr.left, path);
      validateCondition(expr.right, path);
    } else if (expr.type === "not") validateCondition(expr.value, path);
  };
  form.all.forEach((n) => {
    if (n.when) validateCondition(n.when, `${n.key}.when`);
  });
  validateAnswers(form, form.defaults);
  return form;
}
export function validateAnswers(
  form: CompiledForm,
  input: unknown,
): asserts input is Answers {
  const answers = object(input, "answers");
  for (const [id, value] of Object.entries(answers)) {
    const n = form.byId.get(id);
    const p = `answers.${id}`;
    if (!n || !widgets.has(n.kind)) fail(p, "Unknown answer ID");
    if (n.kind === "check" && typeof value !== "boolean")
      fail(p, "Expected a boolean");
    if (n.kind === "input") string(value, p);
    if (n.kind === "slider") {
      const v = number(value, p);
      if (v < n.min! || v > n.max!)
        fail(p, `Expected a number between ${n.min} and ${n.max}`);
    }
    if (
      n.kind === "select" &&
      value !== null &&
      (typeof value !== "string" || !n.options.some((o) => o.value === value))
    )
      fail(p, "Expected null or an option value");
    if (
      ["multi", "rank"].includes(n.kind) &&
      (!Array.isArray(value) ||
        Array.from(value).some(
          (v) => typeof v !== "string" || !n.options.some((o) => o.value === v),
        ) ||
        new Set(value).size !== value.length)
    )
      fail(p, "Expected an array of unique option values");
  }
}
export function computeVisibility(
  form: CompiledForm,
  answers: Answers,
): Set<string> {
  const visible = new Set<string>(),
    active: Answers = Object.create(null);
  const values = { ...form.defaults, ...answers };
  for (const n of form.topo) {
    if (n.parent && !visible.has(n.parent)) continue;
    if (n.gate) {
      const v = active[n.gate.id];
      if (
        n.gate.option !== undefined
          ? !(Array.isArray(v)
              ? v.includes(n.gate.option)
              : v === n.gate.option)
          : !truthy(v)
      )
        continue;
    }
    if (n.when && !truthy(evaluate(n.when, active))) continue;
    visible.add(n.key);
    if (n.id && widgets.has(n.kind)) active[n.id] = values[n.id] ?? null;
  }
  return visible;
}
export function serializeResult(
  form: CompiledForm,
  answers: Answers,
  notes: string,
  visitedPages?: Set<string>,
): PopupResult {
  validateAnswers(form, answers);
  string(notes, "notes");
  const visible = computeVisibility(form, answers);
  const labels: Record<string, string> = Object.create(null);
  const optionLabels: Record<string, Record<string, string>> = Object.create(
    null,
  );
  const result: Record<string, unknown> = {
    status: "completed",
    button: "submit",
  };
  for (const n of form.all)
    if (
      n.id &&
      widgets.has(n.kind) &&
      visible.has(n.key) &&
      (!n.page || !visitedPages || visitedPages.has(n.page))
    ) {
      result[n.id] = Object.hasOwn(answers, n.id)
        ? answers[n.id]
        : form.defaults[n.id];
      labels[n.id] = n.label;
      if (n.options.some((option) => option.stable)) {
        const value = result[n.id];
        const selected = Array.isArray(value)
          ? value
          : typeof value === "string"
            ? [value]
            : [];
        if (selected.length) {
          const mapping: Record<string, string> = Object.create(null);
          for (const id of selected) {
            const option = n.options.find((option) => option.value === id)!;
            mapping[id] = option.label ?? option.value;
          }
          optionLabels[n.id] = mapping;
        }
      }
    }
  result._labels = labels;
  if (Object.keys(optionLabels).length) result._option_labels = optionLabels;
  if (form.notes && notes.trim()) result._notes = notes.trim();
  return result as PopupResult;
}

/** Snap interactions to min-based steps, allowing max as a reachable endpoint.
 * Explicit authored defaults are preserved; only implicit defaults and interactions snap.
 */
export function snapSliderValue(node: Node, value: number): number {
  const min = node.min!,
    max = node.max!,
    step = node.step ?? 1;
  const clamped = Math.min(max, Math.max(min, value));
  const position = (clamped - min) / step;
  if (!Number.isFinite(position)) return clamped;
  const candidates = [
    min + Math.floor(position) * step,
    min + Math.ceil(position) * step,
    max,
  ].filter((v) => Number.isFinite(v) && v >= min && v <= max);
  let best = min,
    distance = Infinity;
  for (const candidate of candidates) {
    const delta = Math.abs(candidate - clamped);
    const tolerance =
      Number.EPSILON * Math.max(1, Math.abs(clamped), Math.abs(candidate)) * 4;
    if (
      delta < distance - tolerance ||
      (Math.abs(delta - distance) <= tolerance && candidate > best)
    ) {
      best = candidate;
      distance = delta;
    }
  }
  // Remove arithmetic noise for decimal steps without changing the range endpoints.
  return best === min || best === max
    ? best
    : Math.min(max, Math.max(min, Number(best.toPrecision(15))));
}

/** Maintains visibility by reevaluating changed answers and their dependent nodes. */
export function createVisibilityEvaluator(form: CompiledForm): {
  update(answers: Answers, changedIds?: Iterable<string>): Set<string>;
} {
  const dependents = new Map<string, Set<string>>();
  const order = new Map(form.topo.map((n, i) => [n.key, i]));
  for (const n of form.all) {
    const deps = [
      ...(n.parent ? [n.parent] : []),
      ...n.refs.map((id) => form.byId.get(id)!.key),
    ];
    for (const key of deps) {
      if (!dependents.has(key)) dependents.set(key, new Set());
      dependents.get(key)!.add(n.key);
    }
  }
  let visible = new Set<string>(),
    initialized = false;
  const active: Answers = Object.create(null);
  return {
    update(answers, changedIds) {
      let nodes: Node[];
      if (!initialized || changedIds === undefined) nodes = form.topo;
      else {
        const affected = new Set<string>(),
          queue: string[] = [];
        for (const id of changedIds) {
          const node = form.byId.get(id);
          if (node && !affected.has(node.key)) {
            affected.add(node.key);
            queue.push(node.key);
          }
        }
        for (let i = 0; i < queue.length; i++)
          for (const key of dependents.get(queue[i]) ?? [])
            if (!affected.has(key)) {
              affected.add(key);
              queue.push(key);
            }
        nodes = queue
          .sort((a, b) => order.get(a)! - order.get(b)!)
          .map((key) => form.byKey.get(key)!);
      }
      let next = visible;
      for (const n of nodes) {
        let shown = !n.parent || next.has(n.parent);
        if (shown && n.gate) {
          const value = active[n.gate.id];
          shown =
            n.gate.option === undefined
              ? truthy(value)
              : Array.isArray(value)
                ? value.includes(n.gate.option)
                : value === n.gate.option;
        }
        if (shown && n.when) shown = truthy(evaluate(n.when, active));
        if (shown !== next.has(n.key)) {
          if (next === visible) next = new Set(visible);
          if (shown) next.add(n.key);
          else next.delete(n.key);
        }
        if (n.id && widgets.has(n.kind)) {
          if (shown)
            active[n.id] = Object.hasOwn(answers, n.id)
              ? answers[n.id]
              : form.defaults[n.id];
          else delete active[n.id];
        }
      }
      initialized = true;
      visible = next;
      return visible;
    },
  };
}
