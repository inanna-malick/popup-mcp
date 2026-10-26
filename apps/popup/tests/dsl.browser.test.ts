import { expect, test, type Page } from "@playwright/test";
import type { Envelope } from "../src/bridge/delivery";

type Harness = {
  showForm(definition: unknown): Promise<void>;
  messages: Envelope[];
};
async function show(page: Page, definition: unknown) {
  await page.goto("/tests/host.html");
  await page.evaluate(async (definition) => {
    await (window as unknown as Harness).showForm(definition);
  }, definition);
  return page.frameLocator('[data-testid="app"]');
}
async function result(page: Page) {
  await expect
    .poll(() =>
      page.evaluate(() => (window as unknown as Harness).messages.length),
    )
    .toBe(1);
  return page.evaluate(() => (window as unknown as Harness).messages[0].result);
}

test("recommendations are visible and described without selecting answers or opening branches", async ({
  page,
}) => {
  const app = await show(page, {
    notes: false,
    elements: [
      {
        select: "Approach",
        id: "approach",
        options: [
          {
            id: "repair",
            label: "Repair system",
            description: "Keep the current system",
            recommendation: "Avoid a migration",
          },
          { id: "replace", label: "Replace system" },
        ],
        repair: [{ input: "Repair plan", id: "plan" }],
      },
      {
        multi: "Features",
        id: "features",
        options: [
          {
            id: "logs",
            label: "Operation logs",
            recommendation: "Make diagnosis easier",
          },
        ],
      },
      {
        select: "Mode",
        id: "mode",
        style: "dropdown",
        options: [
          {
            id: "careful",
            label: "Careful mode",
            recommendation: "Verify each change",
          },
        ],
      },
      {
        rank: "Priorities",
        id: "priorities",
        options: [
          {
            id: "reliability",
            label: "Reliable operation",
            recommendation: "Reduce interruptions",
          },
          { id: "speed", label: "Fast operation" },
        ],
      },
    ],
  });
  await expect(
    app.getByText("Avoid a migration", { exact: false }),
  ).toBeVisible();
  await expect(
    app.getByText("Make diagnosis easier", { exact: false }),
  ).toBeVisible();
  await expect(
    app.getByText("Verify each change", { exact: false }),
  ).toBeVisible();
  await expect(
    app.getByText("Reduce interruptions", { exact: false }),
  ).toBeVisible();
  const repair = app.getByRole("radio", { name: "Repair system", exact: true });
  await expect(repair).toHaveAccessibleDescription(
    "Keep the current system Recommended — Avoid a migration",
  );
  await expect(repair).not.toBeChecked();
  await expect(
    app.getByRole("checkbox", { name: "Operation logs", exact: true }),
  ).not.toBeChecked();
  await expect(app.getByLabel("Mode", { exact: true })).toHaveValue("");
  await expect(
    app.getByLabel("Mode", { exact: true }),
  ).toHaveAccessibleDescription("Recommended — Verify each change");
  await expect(
    app.getByRole("option", {
      name: "Careful mode (Recommended)",
      exact: true,
    }),
  ).toHaveCount(1);
  await expect(
    app.getByRole("button", { name: "Reliable operation", exact: true }),
  ).toHaveAttribute("aria-pressed", "false");
  await expect(app.getByLabel("Repair plan")).toHaveCount(0);
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  expect(await result(page)).toMatchObject({
    approach: null,
    features: [],
    mode: null,
    priorities: [],
  });
});

test("display label changes preserve IDs and branches while summaries use submitted labels", async ({
  page,
}) => {
  for (const label of ["Repair system", "Preserve the existing system"]) {
    const app = await show(page, {
      title: "Stable choice labels",
      notes: false,
      elements: [
        {
          select: "Approach",
          id: "approach",
          options: [{ id: "repair", label }],
          repair: [{ input: "Plan", id: "plan" }],
        },
        {
          multi: "Features",
          id: "features",
          options: [{ id: "logs", label: "Operation logs" }],
        },
        {
          select: "Mode",
          id: "mode",
          style: "dropdown",
          options: [{ id: "careful", label: "Careful mode" }],
        },
        {
          rank: "Priorities",
          id: "priorities",
          options: [
            { id: "uptime", label: "Service availability" },
            { id: "latency", label: "Low latency" },
          ],
        },
        {
          rank: "Legacy names",
          id: "legacy",
          options: ["toString", "constructor", "__proto__"],
        },
      ],
    });
    await app.getByRole("radio", { name: label, exact: true }).check();
    await app
      .getByLabel("Plan", { exact: true })
      .fill("Reuse the current implementation");
    await app
      .getByRole("checkbox", { name: "Operation logs", exact: true })
      .check();
    await app.getByLabel("Mode", { exact: true }).selectOption("careful");
    await app
      .getByRole("button", { name: "Service availability", exact: true })
      .click();
    for (const name of ["toString", "constructor", "__proto__"])
      await app.getByRole("button", { name, exact: true }).click();
    await app.getByRole("button", { name: "Submit", exact: true }).click();
    const submitted = await result(page);
    expect(submitted).toMatchObject({
      approach: "repair",
      plan: "Reuse the current implementation",
      features: ["logs"],
      mode: "careful",
      priorities: ["uptime"],
      _option_labels: {
        approach: { repair: label },
        features: { logs: "Operation logs" },
        mode: { careful: "Careful mode" },
        priorities: { uptime: "Service availability" },
      },
    });
    await expect(app.locator("dl.summary")).toContainText(label);
    await expect(app.locator("dl.summary")).toContainText("Operation logs");
    await expect(app.locator("dl.summary")).toContainText("Careful mode");
    await expect(app.locator("dl.summary")).toContainText(
      "Service availability",
    );
    expect(
      JSON.parse(await app.getByLabel("Submitted response data").inputValue()),
    ).toEqual(submitted);
    await expect(app.locator("dl.summary")).toContainText(
      "toString, constructor, __proto__",
    );
  }
});

test("native disclosure keyboard toggles retain edits and closed answers remain submitted", async ({
  page,
}) => {
  const app = await show(page, {
    notes: false,
    elements: [
      {
        disclosure: "Advanced details",
        elements: [{ input: "Advanced answer", id: "advanced" }],
      },
      {
        disclosure: "Open details",
        open: true,
        elements: [{ input: "Other answer", id: "other" }],
      },
    ],
  });
  const summary = app
    .locator("summary")
    .filter({ hasText: "Advanced details" });
  await expect(summary).toBeFocused();
  await expect(app.getByLabel("Advanced answer")).toHaveCount(1);
  await expect(app.getByLabel("Advanced answer")).toBeHidden();
  await expect(app.getByLabel("Other answer")).toBeVisible();
  await summary.press("Enter");
  await app.getByLabel("Advanced answer").fill("Keep this answer");
  await summary.press("Space");
  await expect(app.getByLabel("Advanced answer")).toBeHidden();
  await expect(app.getByLabel("Other answer")).toBeVisible();
  await summary.press("Enter");
  await expect(app.getByLabel("Advanced answer")).toHaveValue(
    "Keep this answer",
  );
  await summary.press("Enter");
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  expect(await result(page)).toMatchObject({
    advanced: "Keep this answer",
    other: "",
  });
});

test("when hides disclosure answers and restores retained edits when shown", async ({
  page,
}) => {
  const app = await show(page, {
    notes: false,
    elements: [
      { check: "Include details", id: "include", default: true },
      {
        disclosure: "Details",
        when: "include",
        open: true,
        elements: [{ input: "Detail answer", id: "detail" }],
      },
    ],
  });
  await app.getByLabel("Detail answer").fill("Retained edit");
  await app.getByLabel("Include details").uncheck();
  await expect(app.locator(".disclosure")).toHaveCount(0);
  await app.getByLabel("Include details").check();
  await expect(app.getByLabel("Detail answer")).toHaveValue("Retained edit");
  await app.getByLabel("Include details").uncheck();
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  const submitted = await result(page);
  expect(submitted).toMatchObject({ include: false });
  expect(submitted).not.toHaveProperty("detail");
});

test("titleless page navigation and focus recovery skip closed disclosure descendants", async ({
  page,
}) => {
  const app = await show(page, {
    notes: false,
    elements: [
      { page: "Start", elements: [{ input: "First answer", id: "first" }] },
      {
        page: "Finish",
        elements: [
          {
            disclosure: "Optional details",
            elements: [{ input: "Closed answer", id: "closed" }],
          },
          {
            check: "Show answer",
            id: "show",
            default: true,
            reveals: [{ input: "Visible answer", id: "visible" }],
          },
        ],
      },
    ],
  });
  await app.getByRole("button", { name: "Next", exact: true }).click();
  const summary = app
    .locator("summary")
    .filter({ hasText: "Optional details" });
  await expect(summary).toBeFocused();
  await expect(app.getByLabel("Closed answer")).toBeHidden();
  await app.getByLabel("Visible answer").focus();
  await app
    .getByLabel("Show answer")
    .evaluate((element) => (element as HTMLInputElement).click());
  await expect(app.getByLabel("Show answer")).toBeFocused();
  await expect(app.getByLabel("Closed answer")).not.toBeFocused();
});
