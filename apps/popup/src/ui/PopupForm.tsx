import {
  For,
  Show,
  createMemo,
  createSignal,
  createEffect,
  batch,
  onMount,
} from "solid-js";
import { createStore } from "solid-js/store";
import { marked } from "marked";
import DOMPurify from "dompurify";
import {
  compileDefinition,
  createVisibilityEvaluator,
  serializeResult,
  snapSliderValue,
} from "../core";
import type { Answers, CompiledForm, Node, PopupResult } from "../core";
import "./popup.css";

export interface PopupFormProps {
  definition: unknown;
  onSubmit: (result: PopupResult) => Promise<void>;
  onCancel: () => Promise<void>;
  initiallyCompleted?: boolean;
  onOpenLink?: (url: string) => Promise<void>;
}

function formatAnswer(
  value: unknown,
  labels: Record<string, string> = {},
): string {
  const display = (item: unknown) =>
    Object.hasOwn(labels, String(item)) ? labels[String(item)] : String(item);
  if (typeof value === "boolean") return value ? "Yes" : "No";
  if (Array.isArray(value))
    return value.length ? value.map(display).join(", ") : "None selected";
  return value == null || value === "" ? "No answer" : display(value);
}

function isVisibleControl(element: HTMLElement): boolean {
  if (!element.isConnected || !element.getClientRects().length) return false;
  for (
    let parent = element.parentElement;
    parent;
    parent = parent.parentElement
  ) {
    if (parent instanceof HTMLDetailsElement && !parent.open) {
      const summary = parent.querySelector(":scope > summary");
      if (!summary?.contains(element)) return false;
    }
  }
  return getComputedStyle(element).visibility !== "hidden";
}

function visibleControl(
  region: HTMLElement | undefined,
): HTMLElement | undefined {
  return Array.from(
    region?.querySelectorAll<HTMLElement>(
      "input:not(:disabled), textarea:not(:disabled), select:not(:disabled), button:not(:disabled), summary",
    ) || [],
  ).find(isVisibleControl);
}

function freezeResult(result: PopupResult): PopupResult {
  // Answer arrays may be Solid store proxies; freeze a detached wire snapshot.
  const snapshot = JSON.parse(JSON.stringify(result)) as PopupResult;
  const freeze = (value: unknown): void => {
    if (value && typeof value === "object") {
      for (const child of Object.values(value)) freeze(child);
      Object.freeze(value);
    }
  };
  freeze(snapshot);
  return snapshot;
}

export function PopupForm(props: PopupFormProps) {
  let form: CompiledForm;
  try {
    form = compileDefinition(props.definition);
  } catch (error) {
    const [closeError, setCloseError] = createSignal("");
    return (
      <main class="popup">
        <h1>Unable to open this form</h1>
        <pre role="alert">{String(error)}</pre>
        <Show when={closeError()}>
          <p role="alert">{closeError()}</p>
        </Show>
        <button
          onClick={async () => {
            try {
              await props.onCancel();
            } catch (reason) {
              setCloseError(String(reason));
            }
          }}
        >
          Close
        </button>
      </main>
    );
  }
  const compiled = form;
  const [answers, setAnswers] = createStore<Answers>({ ...compiled.defaults });
  const [numericDrafts, setNumericDrafts] = createStore<
    Record<string, string | undefined>
  >({});
  const [numericErrors, setNumericErrors] = createStore<
    Record<string, string | undefined>
  >({});
  const [notes, setNotes] = createSignal("");
  const [copied, setCopied] = createSignal(false);
  const [copyError, setCopyError] = createSignal("");
  const [state, setState] = createSignal<
    "editing" | "sending" | "done" | "error"
  >(props.initiallyCompleted ? "done" : "editing");
  const [error, setError] = createSignal("");
  const [cancelled, setCancelled] = createSignal(false);
  const [snapshot, setSnapshot] = createSignal<PopupResult>();
  const resultData = () => snapshot() as Record<string, unknown> | undefined;
  const [pageIndex, setPageIndex] = createSignal(0);
  const visited = new Set<string>();
  const evaluator = createVisibilityEvaluator(compiled);
  const [visible, setVisible] = createSignal(evaluator.update(answers));
  const pages = createMemo(() =>
    compiled.all.filter(
      (node) => node.kind === "page" && visible().has(node.key),
    ),
  );
  const activePage = createMemo(
    () => pages()[Math.min(pageIndex(), Math.max(0, pages().length - 1))],
  );
  const locked = () => state() !== "editing";
  let main: HTMLElement | undefined;
  let heading: HTMLHeadingElement | undefined;
  let status: HTMLDivElement | undefined;
  createEffect(() => {
    const last = Math.max(0, pages().length - 1);
    if (pageIndex() > last) setPageIndex(last);
  });
  createEffect(() => {
    const page = activePage();
    if (page) visited.add(page.key);
  });
  let initialPage = true;
  createEffect(() => {
    pageIndex();
    if (initialPage) {
      initialPage = false;
      return;
    }
    queueMicrotask(() =>
      (heading || firstControl() || main)?.focus({ preventScroll: true }),
    );
  });
  const firstControl = () => visibleControl(main);
  onMount(() => (firstControl() || main)?.focus({ preventScroll: true }));
  createEffect(() => {
    if (state() === "done" || state() === "error")
      queueMicrotask(() => status?.focus());
  });

  const deliver = async (result: PopupResult) => {
    setState("sending");
    setError("");
    try {
      if (result.status === "cancelled") await props.onCancel();
      else await props.onSubmit(result);
      setCancelled(result.status === "cancelled");
      setState("done");
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
      setState("error");
    }
  };
  const submit = async () => {
    if (state() === "sending" || state() === "done") return;
    if (state() === "editing" && !commitNumbers()) return;
    if (state() === "editing" && pageIndex() < pages().length - 1) {
      setPageIndex((index) => index + 1);
      return;
    }
    try {
      let result = snapshot();
      if (!result) {
        result = freezeResult(
          serializeResult(compiled, answers, notes(), visited),
        );
        setSnapshot(result);
      }
      await deliver(result);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  };
  const cancel = async () => {
    if (state() !== "editing") return;
    const result = freezeResult({ status: "cancelled" });
    setSnapshot(result);
    await deliver(result);
  };
  const openLink = async (event: MouseEvent) => {
    const target =
      event.target instanceof Element ? event.target.closest("a") : null;
    if (!target) return;
    event.preventDefault();
    const href = target.getAttribute("href");
    if (!href || !/^https?:\/\//i.test(href)) return;
    try {
      if (props.onOpenLink) await props.onOpenLink(href);
    } catch (reason) {
      setError(reason instanceof Error ? reason.message : String(reason));
    }
  };
  const value = (node: Node) => (node.id ? answers[node.id] : null);
  const update = (node: Node, next: Answers[string]) => {
    if (!node.id || locked()) return;
    const focused = document.activeElement as HTMLElement | null;
    batch(() => {
      if (node.kind === "slider") {
        setNumericDrafts(node.key, undefined);
        setNumericErrors(node.key, undefined);
      }
      setAnswers(node.id!, next);
      setVisible(evaluator.update(answers, [node.id!]));
    });
    queueMicrotask(() => {
      if (focused && focused !== document.body && !isVisibleControl(focused)) {
        const region = Array.from(
          main?.querySelectorAll<HTMLElement>("[data-node-key]") || [],
        ).find((element) => element.dataset.nodeKey === node.key);
        (visibleControl(region) || heading || firstControl() || main)?.focus({
          preventScroll: true,
        });
      }
    });
  };
  const commitNumber = (node: Node) => {
    const draft = numericDrafts[node.key];
    if (draft === undefined) return true;
    const number = Number(draft);
    if (
      !draft.trim() ||
      !Number.isFinite(number) ||
      number < node.min! ||
      number > node.max!
    ) {
      setNumericErrors(
        node.key,
        `Enter a number from ${node.min} to ${node.max}.`,
      );
      return false;
    }
    update(node, snapSliderValue(node, number));
    return true;
  };
  const commitNumbers = () => {
    const invalid = compiled.all.find(
      (node) =>
        node.kind === "slider" &&
        visible().has(node.key) &&
        !commitNumber(node),
    );
    if (!invalid) return true;
    let ancestor: Node | undefined = invalid;
    while (ancestor && ancestor.kind !== "page")
      ancestor = ancestor.parent
        ? compiled.byKey.get(ancestor.parent)
        : undefined;
    if (ancestor) setPageIndex(pages().indexOf(ancestor));
    queueMicrotask(() => {
      const input = document.getElementById(`popup-${invalid.key}-number`);
      for (
        let parent = input?.parentElement;
        parent;
        parent = parent.parentElement
      ) {
        if (parent instanceof HTMLDetailsElement) parent.open = true;
      }
      input?.focus();
    });
    return false;
  };
  const toggle = (node: Node, option: string) => {
    const current = Array.isArray(value(node)) ? (value(node) as string[]) : [];
    update(
      node,
      current.includes(option)
        ? current.filter((item) => item !== option)
        : [...current, option],
    );
  };
  const continuationDepth = new Map<string, number>();
  for (const node of compiled.all) {
    continuationDepth.set(
      node.key,
      (node.parent ? (continuationDepth.get(node.parent) ?? 0) : 0) +
        (node.gate ? 1 : 0),
    );
  }
  function Branch(branchProps: { parent: Node; nodes: Node[] }) {
    return (
      <Show when={branchProps.nodes.some((node) => visible().has(node.key))}>
        <div
          class="branch"
          classList={{
            "branch-flat":
              (continuationDepth.get(branchProps.parent.key) ?? 0) >= 2,
          }}
        >
          <Fields nodes={branchProps.nodes} />
        </div>
      </Show>
    );
  }
  // Group adjacent compatible controls once. Resizing only changes CSS, never
  // field identity, draft state, disclosure state, or keyboard order.
  const packingKind = (node: Node): string | undefined => {
    const leaf =
      !node.elements.length &&
      !node.reveals.length &&
      !Object.keys(node.branches).length;
    if (leaf && node.label.length <= 48) {
      if (node.kind === "check") return "checks";
      if (node.kind === "slider" && node.label.length <= 32) return "sliders";
      if (
        node.kind === "slider" ||
        (node.kind === "input" && (node.rows ?? 1) === 1)
      )
        return "parameters";
    }
    if (
      node.kind === "group" &&
      node.elements.length &&
      node.elements.every((child) =>
        ["check", "slider", "input", "select", "multi", "rank"].includes(
          child.kind,
        ),
      )
    )
      return "groups";
  };
  function Fields(props: { nodes: Node[] }) {
    const runs: { kind?: string; nodes: Node[] }[] = [];
    for (const node of props.nodes) {
      const kind = packingKind(node);
      const previous = runs.at(-1);
      if (kind && previous?.kind === kind) previous.nodes.push(node);
      else runs.push({ kind, nodes: [node] });
    }
    return (
      <div class="fields-list">
        <For each={runs}>
          {(run) =>
            run.kind && run.nodes.length > 1 ? (
              <Show when={run.nodes.some((node) => visible().has(node.key))}>
                <div class={`field-run field-run-${run.kind}`}>
                  <For each={run.nodes}>{(node) => <Field node={node} />}</For>
                </div>
              </Show>
            ) : (
              <For each={run.nodes}>{(node) => <Field node={node} />}</For>
            )
          }
        </For>
      </div>
    );
  }
  const childNodes = (node: Node) => (
    <>
      <Fields nodes={node.elements} />
      <Branch parent={node} nodes={node.reveals} />
      <For each={Object.values(node.branches)}>
        {(children) => <Branch parent={node} nodes={children} />}
      </For>
    </>
  );
  function Field(fieldProps: { node: Node }) {
    const node = fieldProps.node;
    const isOther = (item: Node["options"][number]) =>
      (node.branches[item.value] ?? []).some(
        (child) => child.id === `${node.id}_other_text`,
      );
    const branchColumns =
      node.kind === "multi" &&
      node.options.filter(
        (item) =>
          !isOther(item) && (node.branches[item.value]?.length ?? 0) > 0,
      ).length >= 2;
    const compactChoices =
      node.options.length >= 3 &&
      node.options.every(
        (item) =>
          (item.label ?? item.value).length <= 32 &&
          !item.description &&
          !item.recommendation &&
          (node.branches[item.value] ?? []).every(
            (child) => child.id === `${node.id}_other_text`,
          ),
      );
    const controlId = `popup-${node.key}`;
    const descriptionId = (index: number) =>
      `${controlId}-description-${index}`;
    const recommendationId = (index: number) =>
      `${controlId}-recommendation-${index}`;
    const optionDescription = (item: Node["options"][number], index: number) =>
      [
        item.description ? descriptionId(index) : "",
        item.recommendation ? recommendationId(index) : "",
      ]
        .filter(Boolean)
        .join(" ") || undefined;
    const recommendation = (item: Node["options"][number], index: number) => (
      <Show when={item.recommendation}>
        <span id={recommendationId(index)} class="recommendation">
          <strong>Recommended</strong> — {item.recommendation}
        </span>
      </Show>
    );
    const selected = () =>
      Array.isArray(value(node)) ? (value(node) as string[]) : [];
    const rankedOptions = () => [
      ...selected().map((id) =>
        node.options.find((item) => item.value === id)!,
      ),
      ...node.options.filter((item) => !selected().includes(item.value)),
    ];
    const moveRank = (id: string, direction: number) => {
      const order = [...selected()];
      const from = order.indexOf(id),
        to = from + direction;
      if (from < 0 || to < 0 || to >= order.length || locked()) return;
      [order[from], order[to]] = [order[to], order[from]];
      update(node, order);
      queueMicrotask(() =>
        document
          .getElementById(
            `${controlId}-rank-button-${node.options.findIndex((item) => item.value === id)}`,
          )
          ?.focus(),
      );
    };
    const option = (
      item: Node["options"][number],
      index: number,
      multiple: boolean,
    ) => (
      <div class="option" classList={{ "option-other": isOther(item) }}>
        <label
          class="choice"
          classList={{
            selected: multiple
              ? selected().includes(item.value)
              : value(node) === item.value,
          }}
        >
          <input
            aria-label={item.label ?? item.value}
            value={item.value}
            type={multiple ? "checkbox" : "radio"}
            name={controlId}
            checked={
              multiple
                ? selected().includes(item.value)
                : value(node) === item.value
            }
            disabled={locked()}
            aria-describedby={optionDescription(item, index)}
            onChange={() =>
              multiple ? toggle(node, item.value) : update(node, item.value)
            }
          />
          <span class="choice-copy">
            <span class="choice-title">{item.label ?? item.value}</span>
            <Show when={item.description}>
              <span id={descriptionId(index)} class="description">
                {item.description}
              </span>
            </Show>
            {recommendation(item, index)}
          </span>
        </label>
        <Branch parent={node} nodes={node.branches[item.value] || []} />
      </div>
    );
    return (
      <Show
        when={
          visible().has(node.key) &&
          (node.kind !== "page" || activePage()?.key === node.key)
        }
      >
        <div class={`field field-${node.kind}`} data-node-key={node.key}>
          <Show when={node.kind === "text"}>
            <p>{node.label}</p>
          </Show>
          <Show when={node.kind === "markdown"}>
            <div
              class="markdown"
              innerHTML={DOMPurify.sanitize(
                marked.parse(node.label, { async: false }) as string,
                {
                  ALLOWED_TAGS: [
                    "p",
                    "br",
                    "strong",
                    "em",
                    "del",
                    "a",
                    "code",
                    "pre",
                    "blockquote",
                    "ul",
                    "ol",
                    "li",
                    "h1",
                    "h2",
                    "h3",
                    "h4",
                    "hr",
                    "table",
                    "thead",
                    "tbody",
                    "tr",
                    "th",
                    "td",
                  ],
                  ALLOWED_ATTR: ["href", "title"],
                },
              )}
            />
          </Show>
          <Show when={node.kind === "group" || node.kind === "page"}>
            <fieldset>
              <legend>{node.label}</legend>
              {childNodes(node)}
            </fieldset>
          </Show>
          <Show when={node.kind === "disclosure"}>
            <details class="disclosure" open={node.open ?? false}>
              <summary>{node.label}</summary>
              <div class="disclosure-content">{childNodes(node)}</div>
            </details>
          </Show>
          <Show when={node.kind === "check"}>
            <label class="check">
              <input
                id={controlId}
                type="checkbox"
                checked={Boolean(value(node))}
                disabled={locked()}
                onChange={(event) => update(node, event.currentTarget.checked)}
              />
              {node.label}
            </label>
            {childNodes(node)}
          </Show>
          <Show when={node.kind === "input"}>
            <label for={controlId}>{node.label}</label>
            <Show
              when={(node.rows || 1) > 1}
              fallback={
                <input
                  id={controlId}
                  value={String(value(node) ?? "")}
                  placeholder={node.placeholder}
                  disabled={locked()}
                  onInput={(event) => update(node, event.currentTarget.value)}
                />
              }
            >
              <textarea
                id={controlId}
                rows={node.rows}
                value={String(value(node) ?? "")}
                placeholder={node.placeholder}
                disabled={locked()}
                onInput={(event) => update(node, event.currentTarget.value)}
              />
            </Show>
            {childNodes(node)}
          </Show>
          <Show when={node.kind === "slider"}>
            <div class="slider-heading">
              <label for={controlId}>{node.label}</label>
              <input
                id={`${controlId}-number`}
                class="numeric-value"
                type="number"
                inputmode="decimal"
                aria-label={`${node.label} value`}
                aria-invalid={Boolean(numericErrors[node.key])}
                aria-describedby={
                  numericErrors[node.key] ? `${controlId}-error` : undefined
                }
                min={node.min}
                max={node.max}
                step="any"
                style={{
                  width: `${Math.max(4, Math.min(12, String(numericDrafts[node.key] ?? value(node)).length + 2))}ch`,
                }}
                value={
                  numericDrafts[node.key] ??
                  String(value(node) ?? node.min ?? 0)
                }
                disabled={locked()}
                onInput={(event) => {
                  const draft = event.currentTarget.value;
                  setNumericDrafts(node.key, draft);
                  const number = Number(draft);
                  setNumericErrors(
                    node.key,
                    !draft.trim() ||
                      !Number.isFinite(number) ||
                      number < node.min! ||
                      number > node.max!
                      ? `Enter a number from ${node.min} to ${node.max}.`
                      : undefined,
                  );
                }}
                onBlur={() => commitNumber(node)}
                onKeyDown={(event) => {
                  if (event.isComposing) return;
                  if (event.key === "ArrowUp" || event.key === "ArrowDown") {
                    event.preventDefault();
                    const draft = numericDrafts[node.key];
                    const parsed = draft?.trim() ? Number(draft) : NaN;
                    const current = Number.isFinite(parsed)
                      ? parsed
                      : Number(value(node));
                    const step = node.step ?? 1;
                    const position = (current - node.min!) / step;
                    const tolerance =
                      Number.EPSILON * Math.max(1, Math.abs(position)) * 4;
                    const next =
                      node.min! +
                      (event.key === "ArrowUp"
                        ? Math.floor(position + tolerance) + 1
                        : Math.ceil(position - tolerance) - 1) *
                        step;
                    update(node, snapSliderValue(node, next));
                  }
                  if (event.key === "Enter") {
                    event.preventDefault();
                    event.stopPropagation();
                    commitNumber(node);
                  }
                  if (event.key === "Escape") {
                    event.preventDefault();
                    event.stopPropagation();
                    setNumericDrafts(node.key, undefined);
                    setNumericErrors(node.key, undefined);
                    event.currentTarget.value = String(value(node));
                  }
                }}
              />
            </div>
            <input
              id={controlId}
              type="range"
              min={node.min}
              max={node.max}
              step="any"
              style={{
                "--range-progress": `${Math.max(0, Math.min(100, ((Number(value(node) ?? node.min ?? 0) - node.min!) / (node.max! - node.min! || 1)) * 100))}%`,
              }}
              value={Number(value(node) ?? node.min ?? 0)}
              disabled={locked()}
              onInput={(event) => {
                const next = snapSliderValue(
                  node,
                  Number(event.currentTarget.value),
                );
                event.currentTarget.value = String(next);
                update(node, next);
              }}
              onKeyDown={(event) => {
                if (locked() || event.isComposing) return;
                const current = Number(value(node) ?? node.min ?? 0);
                const step = node.step ?? 1;
                const position = (current - node.min!) / step;
                const tolerance =
                  Number.EPSILON * Math.max(1, Math.abs(position)) * 4;
                let next: number;
                switch (event.key) {
                  case "ArrowRight":
                  case "ArrowUp":
                    next =
                      node.min! + (Math.floor(position + tolerance) + 1) * step;
                    break;
                  case "ArrowLeft":
                  case "ArrowDown":
                    next =
                      node.min! + (Math.ceil(position - tolerance) - 1) * step;
                    break;
                  case "PageUp":
                    next = current + step * 10;
                    break;
                  case "PageDown":
                    next = current - step * 10;
                    break;
                  case "Home":
                    next = node.min!;
                    break;
                  case "End":
                    next = node.max!;
                    break;
                  default:
                    return;
                }
                event.preventDefault();
                next = snapSliderValue(node, next);
                event.currentTarget.value = String(next);
                update(node, next);
              }}
            />
            <div class="range-bounds" aria-hidden="true">
              <span>{node.min}</span>
              <span>{node.max}</span>
            </div>
            <Show when={numericErrors[node.key]}>
              <p class="numeric-error" id={`${controlId}-error`} role="alert">
                {numericErrors[node.key]}
              </p>
            </Show>
            {childNodes(node)}
          </Show>
          <Show when={node.kind === "select" || node.kind === "multi"}>
            <fieldset
              aria-label={
                node.kind === "select" && node.style !== "dropdown"
                  ? node.label
                  : undefined
              }
            >
              <legend
                classList={{
                  "choice-heading":
                    node.kind === "select" && node.style !== "dropdown",
                }}
              >
                <span>{node.label}</span>
                <Show
                  when={node.kind === "select" && node.style !== "dropdown"}
                >
                  <button
                    type="button"
                    class="clear-choice"
                    aria-label={`Clear ${node.label}`}
                    style={{
                      visibility: value(node) == null ? "hidden" : "visible",
                    }}
                    disabled={locked() || value(node) == null}
                    onClick={(event) => {
                      const fieldset = event.currentTarget.closest("fieldset");
                      update(node, null);
                      queueMicrotask(() =>
                        fieldset
                          ?.querySelector<HTMLInputElement>(
                            'input[type="radio"]',
                          )
                          ?.focus(),
                      );
                    }}
                  >
                    Clear
                  </button>
                </Show>
              </legend>
              <Show when={node.kind === "multi"}>
                <p class="field-hint">
                  Choose any that apply
                  <span class="selection-count" aria-live="polite">
                    {selected().length} selected
                  </span>
                </p>
              </Show>
              <Show
                when={node.kind === "select" && node.style === "dropdown"}
                fallback={
                  <div
                    class="choices"
                    classList={{
                      "choices-compact": compactChoices,
                      "choices-branches": branchColumns,
                    }}
                    style={{
                      "--choices-wide": Math.min(
                        node.options.length,
                        node.options.length === 4 ? 4 : 3,
                      ),
                    }}
                  >
                    <For each={node.options}>
                      {(item, index) =>
                        option(item, index(), node.kind === "multi")
                      }
                    </For>
                  </div>
                }
              >
                <label class="sr-only" for={controlId}>
                  {node.label}
                </label>
                <select
                  id={controlId}
                  value={String(value(node) ?? "")}
                  disabled={locked()}
                  aria-describedby={
                    node.options
                      .flatMap((item, index) => [
                        ...(item.recommendation
                          ? [recommendationId(index)]
                          : []),
                        ...(item.description && value(node) === item.value
                          ? [descriptionId(index)]
                          : []),
                      ])
                      .join(" ") || undefined
                  }
                  onChange={(event) =>
                    update(node, event.currentTarget.value || null)
                  }
                >
                  <option value="">Choose an option</option>
                  <For each={node.options}>
                    {(item) => (
                      <option value={item.value}>
                        {item.label ?? item.value}
                        {item.recommendation ? " (Recommended)" : ""}
                      </option>
                    )}
                  </For>
                </select>
                <For each={node.options}>
                  {(item, index) => (
                    <>
                      <Show when={item.recommendation}>
                        <p class="dropdown-recommendation">
                          {item.label ?? item.value}:{" "}
                          {recommendation(item, index())}
                        </p>
                      </Show>
                      <Show when={value(node) === item.value}>
                        <Show when={item.description}>
                          <p id={descriptionId(index())} class="description">
                            {item.description}
                          </p>
                        </Show>
                        <Branch
                          parent={node}
                          nodes={node.branches[item.value] || []}
                        />
                      </Show>
                    </>
                  )}
                </For>
              </Show>
              <For each={node.elements}>
                {(child) => <Field node={child} />}
              </For>
              <Branch parent={node} nodes={node.reveals} />
            </fieldset>
          </Show>
          <Show when={node.kind === "rank"}>
            <fieldset>
              <legend>{node.label}</legend>
              <p class="description">
                Select priorities, then move them into order. Select again to
                remove.
              </p>
              <p class="sr-only rank-status" role="status" aria-atomic="true">
                {selected().length
                  ? `Priority order: ${selected()
                      .map(
                        (id, index) =>
                          `${index + 1}. ${node.options.find((item) => item.value === id)?.label ?? id}`,
                      )
                      .join(", ")}`
                  : "No priorities selected"}
              </p>
              <For each={rankedOptions()}>
                {(item) => {
                  const index = () => node.options.indexOf(item);
                  return (
                    <div class="option rank-option">
                      <div class="rank-row">
                        <button
                          id={`${controlId}-rank-button-${index()}`}
                          type="button"
                          class="rank"
                          aria-label={item.label ?? item.value}
                          aria-pressed={selected().includes(item.value)}
                          aria-describedby={[
                            `${controlId}-rank-${index()}`,
                            optionDescription(item, index()),
                          ]
                            .filter(Boolean)
                            .join(" ")}
                          disabled={locked()}
                          onClick={() => toggle(node, item.value)}
                          onKeyDown={(event) => {
                            if (
                              event.altKey &&
                              ["ArrowUp", "ArrowDown"].includes(event.key)
                            ) {
                              event.preventDefault();
                              moveRank(
                                item.value,
                                event.key === "ArrowUp" ? -1 : 1,
                              );
                            }
                          }}
                        >
                          <span class="rank-number">
                            {selected().includes(item.value)
                              ? selected().indexOf(item.value) + 1
                              : "–"}
                          </span>
                          {item.label ?? item.value}
                        </button>
                        <Show when={selected().includes(item.value)}>
                          <div class="rank-moves">
                            <button
                              type="button"
                              aria-label={`Move ${item.label ?? item.value} up`}
                              disabled={
                                locked() || selected().indexOf(item.value) === 0
                              }
                              onClick={() => moveRank(item.value, -1)}
                            >
                              <svg viewBox="0 0 16 16" aria-hidden="true">
                                <path d="m4 10 4-4 4 4" />
                              </svg>
                            </button>
                            <button
                              type="button"
                              aria-label={`Move ${item.label ?? item.value} down`}
                              disabled={
                                locked() ||
                                selected().indexOf(item.value) ===
                                  selected().length - 1
                              }
                              onClick={() => moveRank(item.value, 1)}
                            >
                              <svg viewBox="0 0 16 16" aria-hidden="true">
                                <path d="m4 6 4 4 4-4" />
                              </svg>
                            </button>
                          </div>
                        </Show>
                      </div>
                      <span id={`${controlId}-rank-${index()}`} class="sr-only">
                        {selected().includes(item.value)
                          ? `Priority ${selected().indexOf(item.value) + 1} of ${selected().length}`
                          : "Not ranked"}
                      </span>
                      <Show when={item.description}>
                        <p id={descriptionId(index())} class="description">
                          {item.description}
                        </p>
                      </Show>
                      {recommendation(item, index())}
                      <Branch
                        parent={node}
                        nodes={node.branches[item.value] || []}
                      />
                    </div>
                  );
                }}
              </For>
              <For each={node.elements}>
                {(child) => <Field node={child} />}
              </For>
              <Branch parent={node} nodes={node.reveals} />
            </fieldset>
          </Show>
        </div>
      </Show>
    );
  }
  return (
    <main
      ref={main}
      class="popup"
      tabIndex={-1}
      aria-label={compiled.title.trim() || "Interactive response"}
      onClick={openLink}
      onKeyDown={(event) => {
        if (
          event.defaultPrevented ||
          event.isComposing ||
          event.keyCode === 229 ||
          event.key !== "Enter"
        )
          return;
        const shortcut = (event.ctrlKey || event.metaKey) && !event.altKey;
        const singleLine =
          event.target instanceof HTMLInputElement &&
          event.target.type === "text" &&
          !event.altKey &&
          !event.shiftKey;
        if (shortcut || singleLine) {
          event.preventDefault();
          void submit();
        }
      }}
    >
      <Show when={compiled.title.trim()}>
        <header class="form-header">
          <h1 ref={heading} tabIndex={-1}>
            {compiled.title}
          </h1>
        </header>
      </Show>
      <Show when={state() === "editing" || state() === "sending"}>
        <form
          aria-busy={state() === "sending"}
          onSubmit={(event) => {
            event.preventDefault();
            void submit();
          }}
        >
          <Show when={pages().length > 0}>
            <div class="page-progress">
              <p class="progress" aria-live="polite">
                Page {Math.min(pageIndex() + 1, pages().length)} of{" "}
                {pages().length}
              </p>
              <div class="progress-track" aria-hidden="true">
                <span
                  style={{
                    width: `${((pageIndex() + 1) / pages().length) * 100}%`,
                  }}
                />
              </div>
            </div>
          </Show>
          <Fields nodes={compiled.nodes} />
          <Show when={compiled.notes}>
            <div class="notes">
              <label for="popup-notes">
                Additional notes <span class="optional">Optional</span>
              </label>
              <textarea
                id="popup-notes"
                placeholder="Anything else, or anything we missed?"
                rows={1}
                value={notes()}
                disabled={locked()}
                onInput={(event) => setNotes(event.currentTarget.value)}
              />
            </div>
          </Show>
          <Show when={error()}>
            <p role="alert">{error()}</p>
          </Show>
          <div class="actions">
            <button
              type="button"
              class="quiet"
              disabled={locked()}
              onClick={cancel}
            >
              Cancel
            </button>
            <Show when={pageIndex() > 0}>
              <button
                type="button"
                disabled={locked()}
                onClick={() => setPageIndex((index) => index - 1)}
              >
                Back
              </button>
            </Show>
            <Show
              when={pageIndex() < pages().length - 1}
              fallback={
                <button
                  type="button"
                  class="primary"
                  disabled={locked()}
                  onClick={submit}
                >
                  {state() === "sending" ? "Sending…" : compiled.submit}
                </button>
              }
            >
              <button
                type="button"
                class="primary"
                disabled={locked()}
                onClick={submit}
              >
                Next
              </button>
            </Show>
          </div>
        </form>
      </Show>
      <Show when={state() === "done" || state() === "error"}>
        <div
          ref={status}
          class="completion"
          data-state={state()}
          tabIndex={-1}
          role="status"
        >
          <h2>
            {state() === "done"
              ? cancelled()
                ? "Form cancelled"
                : "Response complete"
              : "Your response could not be delivered"}
          </h2>
          <Show when={state() === "done" && !cancelled()}>
            <p>Your response has been sent to the conversation.</p>
          </Show>
          <Show
            when={state() === "error"}
            fallback={
              <Show when={error()}>
                <p role="alert">{error()}</p>
              </Show>
            }
          >
            <p role="alert">
              Your response is still available below. Retry delivery or copy it
              into the conversation.
            </p>
            <Show when={error()}>
              <details class="delivery-detail">
                <summary>Delivery details</summary>
                <p>{error()}</p>
              </details>
            </Show>
          </Show>
          <div class="recovery-actions">
            <Show when={state() === "error"}>
              <button class="primary" onClick={submit}>
                Retry delivery
              </button>
            </Show>
            <Show when={snapshot()}>
              <button
                onClick={async () => {
                  setCopyError("");
                  try {
                    await navigator.clipboard.writeText(
                      JSON.stringify(snapshot(), null, 2),
                    );
                    setCopied(true);
                  } catch {
                    setCopyError(
                      "Clipboard unavailable. Expand Response data to select and copy your response.",
                    );
                  }
                }}
              >
                {copied() ? "Copied" : "Copy response"}
              </button>
            </Show>
          </div>
          <Show when={copyError()}>
            <p role="alert">{copyError()}</p>
          </Show>
          <Show when={snapshot()}>
            <Show
              when={snapshot()?.status === "completed"}
              fallback={<p>Cancellation requested.</p>}
            >
              <dl class="summary">
                <For
                  each={Object.entries(
                    (resultData()?._labels as Record<string, string>) || {},
                  )}
                >
                  {([id, label]) => (
                    <>
                      <dt>{label}</dt>
                      <dd>
                        {formatAnswer(
                          resultData()?.[id],
                          (
                            resultData()?._option_labels as
                              Record<string, Record<string, string>> | undefined
                          )?.[id],
                        )}
                      </dd>
                    </>
                  )}
                </For>
                <Show when={resultData()?._notes}>
                  <dt>Additional notes</dt>
                  <dd>{String(resultData()?._notes)}</dd>
                </Show>
              </dl>
            </Show>
            <details>
              <summary>Response data</summary>
              <label class="sr-only" for="popup-result">
                Submitted response data
              </label>
              <textarea
                id="popup-result"
                class="result"
                readOnly
                rows={12}
                value={JSON.stringify(snapshot(), null, 2)}
                onFocus={(event) => event.currentTarget.select()}
              />
            </details>
          </Show>
        </div>
      </Show>
    </main>
  );
}
