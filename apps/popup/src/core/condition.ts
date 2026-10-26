import type { Answers, Expr } from "./types";
export const truthy = (v: unknown): boolean =>
  Array.isArray(v) ? v.length > 0 : Boolean(v);
export function parseCondition(source: string): { expr: Expr; refs: string[] } {
  if (source.length > 4096)
    throw new Error("Condition exceeds 4096 characters");
  let pos = 0,
    depth = 0;
  const refs = new Set<string>();
  const fail = (message: string): never => {
    throw new Error(`${message} at character ${pos + 1}`);
  };
  const ws = () => {
    while (/\s/.test(source[pos] ?? "") && pos < source.length) pos++;
  };
  const take = (s: string) => {
    ws();
    if (source.startsWith(s, pos)) {
      pos += s.length;
      return true;
    }
    return false;
  };
  const expect = (s: string) => {
    if (!take(s)) fail(`Expected ${s}`);
  };
  const quoted = (): string => {
    ws();
    const quote = source[pos++];
    let value = "";
    while (pos < source.length) {
      const c = source[pos++];
      if (c === quote) return value;
      if (c !== "\\") {
        value += c;
        continue;
      }
      const escaped = source[pos++];
      if (escaped === "u") {
        const hex = source.slice(pos, pos + 4);
        if (!/^[0-9a-fA-F]{4}$/.test(hex)) fail("Invalid Unicode escape");
        value += String.fromCharCode(parseInt(hex, 16));
        pos += 4;
      } else {
        const escapes: Record<string, string> = {
          n: "\n",
          r: "\r",
          t: "\t",
          b: "\b",
          f: "\f",
          "\\": "\\",
          '"': '"',
          "'": "'",
          "/": "/",
        };
        if (!(escaped in escapes)) fail("Invalid string escape");
        value += escapes[escaped];
      }
    }
    return fail("Unterminated string");
  };
  const primary = (): Expr => {
    if (++depth > 64) fail("Condition nesting exceeds 64");
    try {
      ws();
      if (take("!")) return { type: "not", value: primary() };
      if (take("(")) {
        const e = or();
        expect(")");
        return e;
      }
      if (source[pos] === '"' || source[pos] === "'")
        return { type: "literal", value: quoted() };
      const number = /^-?\d+(?:\.\d+)?/.exec(source.slice(pos));
      if (number) {
        pos += number[0].length;
        const value = Number(number[0]);
        if (!Number.isFinite(value)) fail("Number must be finite");
        return { type: "literal", value };
      }
      const id = /^@?([a-zA-Z_][a-zA-Z0-9_]*)/.exec(source.slice(pos));
      if (!id) return fail("Expected a value");
      pos += id[0].length;
      const name = id[1];
      if (!id[0].startsWith("@") && (name === "true" || name === "false"))
        return { type: "literal", value: name === "true" };
      if (take("(")) {
        if (!["selected", "count", "any", "all"].includes(name))
          fail(`Unknown function ${name}; use selected, count, any, or all`);
        const args: Expr[] = [];
        if (name === "selected") {
          args.push(primary());
          if (args[0].type !== "ref")
            fail("selected first argument must be a field ID");
          expect(",");
          ws();
          let value: string;
          if (source[pos] === '"' || source[pos] === "'") value = quoted();
          else {
            const end = source.indexOf(")", pos);
            if (end < 0) fail("Expected )");
            value = source.slice(pos, end).trim();
            pos = end;
            if (!value || /[(),]/.test(value)) fail("Invalid option value");
          }
          args.push({ type: "literal", value });
          expect(")");
        } else {
          if (!take(")")) {
            do {
              args.push(or());
            } while (take(","));
            expect(")");
          }
          if (name === "count" && (args.length !== 1 || args[0].type !== "ref"))
            fail("count requires one field ID");
          if ((name === "any" || name === "all") && args.length === 0)
            fail(`${name} requires at least one argument`);
        }
        return { type: "call", name, args };
      }
      refs.add(name);
      return { type: "ref", id: name };
    } finally {
      depth--;
    }
  };
  const comparison = (): Expr => {
    const left = primary();
    ws();
    const op = /^(>=|<=|==|!=|>|<)/.exec(source.slice(pos));
    if (!op) return left;
    pos += op[0].length;
    return { type: "binary", op: op[0], left, right: primary() };
  };
  const and = (): Expr => {
    let e = comparison();
    while (take("&&"))
      e = { type: "binary", op: "&&", left: e, right: comparison() };
    return e;
  };
  const or = (): Expr => {
    let e = and();
    while (take("||")) e = { type: "binary", op: "||", left: e, right: and() };
    return e;
  };
  const expr = or();
  ws();
  if (pos !== source.length) fail("Unexpected token");
  return { expr, refs: [...refs] };
}
export function evaluate(expr: Expr, answers: Answers): Answers[string] {
  switch (expr.type) {
    case "literal":
      return expr.value;
    case "ref":
      return answers[expr.id] ?? null;
    case "not":
      return !truthy(evaluate(expr.value, answers));
    case "call": {
      const values = expr.args.map((e) => evaluate(e, answers));
      if (expr.name === "any") return values.some(truthy);
      if (expr.name === "all") return values.every(truthy);
      if (expr.name === "count")
        return Array.isArray(values[0])
          ? values[0].filter((v) => v.length > 0).length
          : typeof values[0] === "boolean"
            ? Number(values[0])
            : typeof values[0] === "number" && values[0] > 0
              ? 1
              : 0;
      if (typeof values[0] === "boolean")
        return (
          values[0] &&
          expr.args[0].type === "ref" &&
          expr.args[0].id === values[1]
        );
      return Array.isArray(values[0])
        ? values[0].includes(String(values[1]))
        : values[0] === values[1];
    }
    case "binary": {
      const l = evaluate(expr.left, answers);
      if (expr.op === "&&")
        return truthy(l) && truthy(evaluate(expr.right, answers));
      if (expr.op === "||")
        return truthy(l) || truthy(evaluate(expr.right, answers));
      const r = evaluate(expr.right, answers);
      const equal = JSON.stringify(l) === JSON.stringify(r);
      if (expr.op === "==") return equal;
      if (expr.op === "!=") return !equal;
      if (!(
        (typeof l === "number" && typeof r === "number") ||
        (typeof l === "string" && typeof r === "string")
      ))
        return false;
      switch (expr.op) {
        case ">":
          return l > r;
        case "<":
          return l < r;
        case ">=":
          return l >= r;
        default:
          return l <= r;
      }
    }
  }
}
