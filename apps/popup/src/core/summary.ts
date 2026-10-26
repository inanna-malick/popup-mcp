import type { PopupResult } from "./types";

/** One answered field, as a person would read it. */
export interface SummaryRow {
  id: string;
  label: string;
  value: string;
  /** The user left the authored default in place. */
  defaulted: boolean;
}

export function formatAnswer(
  value: unknown,
  labels: Record<string, string> = {},
): string {
  const display = (item: unknown) =>
    Object.hasOwn(labels, String(item)) ? labels[String(item)] : String(item);
  if (typeof value === "boolean") return value ? "Yes" : "No";
  if (Array.isArray(value))
    return value.length ? value.map(display).join(", ") : "None selected";
  return display(value);
}

function unanswered(value: unknown, defaulted: boolean): boolean {
  if (value == null || value === "") return true;
  // An empty selection nobody touched says nothing; one the user cleared is an answer.
  return Array.isArray(value) && value.length === 0 && defaulted;
}

/** The answered fields of a completed result, in display order. Unanswered fields are left out. */
export function summaryRows(result: PopupResult): SummaryRow[] {
  if (result.status !== "completed") return [];
  const defaulted = new Set(result._defaulted ?? []);
  return Object.entries(result._labels)
    .filter(([id]) => !unanswered(result[id], defaulted.has(id)))
    .map(([id, label]) => ({
      id,
      label,
      value: formatAnswer(result[id], result._option_labels?.[id]),
      defaulted: defaulted.has(id),
    }));
}

/**
 * The text of the user's turn: readable in a transcript, and complete enough for a model to act
 * on even if the host drops the structured envelope. The response id lets a model recognize a
 * duplicate delivery.
 */
export function summaryText(
  result: PopupResult,
  { title, submissionId }: { title?: string; submissionId: string },
): string {
  const heading = title?.trim() ? ` · ${title.trim()}` : "";
  if (result.status !== "completed")
    return `Popup cancelled${heading}\n(response ${submissionId})`;
  const lines = summaryRows(result).map(
    (row) => `- ${row.label}: ${row.value}${row.defaulted ? " (default)" : ""}`,
  );
  if (result._notes) lines.push(`Notes: ${result._notes}`);
  if (!lines.length) lines.push("(no answers given)");
  return [
    `Popup response${heading}`,
    ...lines,
    `(response ${submissionId})`,
  ].join("\n");
}
