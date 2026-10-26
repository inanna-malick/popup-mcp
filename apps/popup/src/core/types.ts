export type Answers = Record<
  string,
  string | number | boolean | string[] | null
>;
export type Expr =
  | { type: "literal"; value: Answers[string] }
  | { type: "ref"; id: string }
  | { type: "not"; value: Expr }
  | { type: "binary"; op: string; left: Expr; right: Expr }
  | { type: "call"; name: string; args: Expr[] };
export interface Node {
  key: string;
  id?: string;
  kind:
    | "text"
    | "markdown"
    | "slider"
    | "check"
    | "input"
    | "select"
    | "multi"
    | "rank"
    | "disclosure"
    | "group"
    | "page";
  label: string;
  options: {
    value: string;
    label?: string;
    stable?: true;
    description?: string;
    recommendation?: string;
  }[];
  elements: Node[];
  reveals: Node[];
  branches: Record<string, Node[]>;
  min?: number;
  max?: number;
  step?: number;
  rows?: number;
  placeholder?: string;
  style?: "radio" | "dropdown";
  open?: boolean;
  when?: Expr;
  refs: string[];
  parent?: string;
  gate?: { id: string; option?: string };
  page?: string;
}
export interface CompiledForm {
  title: string;
  submit: string;
  notes: boolean;
  nodes: Node[];
  defaults: Answers;
  all: Node[];
  byId: Map<string, Node>;
  byKey: Map<string, Node>;
  topo: Node[];
}
export type PopupResult =
  | { status: "cancelled" }
  | ({
      status: "completed";
      _labels: Record<string, string>;
      _notes?: string;
      _option_labels?: Record<string, Record<string, string>>;
      /** Visible fields submitted with the authored default, never changed by the user. */
      _defaulted?: string[];
    } & Record<string, unknown>);
export interface ValidationIssue {
  path: string;
  message: string;
}
export class DefinitionError extends Error {
  constructor(public issues: ValidationIssue[]) {
    super(issues.map((i) => `${i.path}: ${i.message}`).join("\n"));
    this.name = "DefinitionError";
  }
}
