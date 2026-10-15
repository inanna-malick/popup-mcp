import { expect, test, type Page } from "@playwright/test";
import type { Envelope } from "../src/bridge/delivery";

type Harness = {
  showForm(
    definition: unknown,
    options?: {
      rejectMessages?: number;
      messaging?: boolean;
      formId?: string;
      theme?: "light" | "dark";
    },
  ): Promise<void>;
  messages: Envelope[];
  attempts: Envelope[];
};
async function show(
  page: Page,
  definition: unknown,
  options: Parameters<Harness["showForm"]>[1] = {},
) {
  await page.goto("/tests/host.html");
  await page.evaluate(
    async ({ definition, options }) => {
      await (window as unknown as Harness).showForm(definition, options);
    },
    { definition, options },
  );
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

test("reveals nested branches, retains hidden edits, and becomes immutable after submit", async ({
  page,
}) => {
  const app = await show(page, {
    title: "Choose a fix",
    elements: [
      { input: "Always available", id: "stable" },
      {
        check: "Advanced",
        id: "advanced",
        reveals: [
          {
            select: "Approach",
            id: "approach",
            options: [
              { value: "Repair", description: "Keep the existing system" },
              "Replace",
            ],
            Repair: [
              {
                check: "Add tests",
                id: "tests",
                reveals: [{ input: "Test plan", id: "plan" }],
              },
            ],
          },
        ],
      },
    ],
  });
  await expect(app.getByLabel("Always available")).toBeFocused();
  await app.getByLabel("Always available").fill("Keep this");
  await app.getByLabel("Advanced", { exact: true }).check();
  await app.getByRole("radio", { name: "Repair", exact: true }).check();
  await expect(app.getByText("Keep the existing system")).toBeVisible();
  await app.getByLabel("Add tests", { exact: true }).check();
  await app.getByLabel("Test plan").fill("Integration coverage");
  await app.getByLabel("Advanced", { exact: true }).uncheck();
  await expect(app.getByLabel("Test plan")).toHaveCount(0);
  await expect(app.getByLabel("Always available")).toHaveValue("Keep this");
  await app.getByLabel("Advanced", { exact: true }).check();
  await expect(app.getByLabel("Test plan")).toHaveValue("Integration coverage");
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  expect(await result(page)).toMatchObject({
    stable: "Keep this",
    advanced: true,
    approach: "Repair",
    tests: true,
    plan: "Integration coverage",
  });
  await expect(
    app.getByRole("heading", { name: "Response complete" }),
  ).toBeVisible();
  await expect(app.getByLabel("Always available")).toHaveCount(0);
  await expect(app.locator("dl")).toContainText("Integration coverage");
  await expect(
    app.getByRole("button", { name: "Submit", exact: true }),
  ).toHaveCount(0);
});

test("failed cancellation retries the same cancellation without reopening fields", async ({
  page,
}) => {
  const app = await show(
    page,
    { title: "Cancel safely", elements: [{ input: "Name" }] },
    { rejectMessages: 1 },
  );
  await app.getByRole("button", { name: "Cancel", exact: true }).click();
  await expect(
    app.getByRole("button", { name: "Retry delivery" }),
  ).toBeVisible();
  await expect(app.getByLabel("Name", { exact: true })).toHaveCount(0);
  await expect(
    app.getByRole("button", { name: "Submit", exact: true }),
  ).toHaveCount(0);
  await app.getByRole("button", { name: "Retry delivery" }).click();
  expect(await result(page)).toEqual({ status: "cancelled" });
  const attempts = await page.evaluate(
    () => (window as unknown as Harness).attempts,
  );
  expect(attempts).toHaveLength(2);
  expect(attempts[1]).toEqual(attempts[0]);
  await expect(
    app.getByRole("heading", { name: "Form cancelled" }),
  ).toBeVisible();
});

test("dropdown clears to null and rank toggles preserve selection order", async ({
  page,
}) => {
  const app = await show(page, {
    title: "Priorities",
    elements: [
      {
        select: "Mode",
        id: "mode",
        style: "dropdown",
        default: "Fast",
        options: ["Fast", "Careful"],
      },
      {
        rank: "Priority",
        id: "priority",
        options: ["Reliability", "Speed", "Cost"],
      },
    ],
  });
  await app.getByLabel("Mode", { exact: true }).selectOption("");
  await app.getByRole("button", { name: "Speed", exact: true }).click();
  await app.getByRole("button", { name: "Reliability", exact: true }).click();
  await app.getByRole("button", { name: "Cost", exact: true }).click();
  await app.getByRole("button", { name: "Cost", exact: true }).click();
  await expect(
    app.getByRole("button", { name: "Speed", exact: true }),
  ).toHaveAttribute("aria-pressed", "true");
  await expect(
    app.getByRole("button", { name: "Cost", exact: true }),
  ).toHaveAttribute("aria-pressed", "false");
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  expect(await result(page)).toMatchObject({
    mode: null,
    priority: ["Speed", "Reliability"],
  });
});

test("linear navigation skips hidden pages and preserves multiline Enter", async ({
  page,
}) => {
  const app = await show(page, {
    title: "Two steps",
    elements: [
      {
        page: "Start",
        elements: [
          { check: "Extra page", id: "extra" },
          { input: "First answer", id: "first" },
        ],
      },
      {
        page: "Optional",
        when: "extra",
        elements: [{ input: "Skipped answer", id: "skipped" }],
      },
      {
        page: "Finish",
        elements: [{ input: "Detailed answer", id: "details", rows: 3 }],
      },
    ],
  });
  await expect(app.getByText("Page 1 of 2")).toBeVisible();
  await app.getByLabel("First answer").fill("Ready");
  await app.getByLabel("First answer").press("Control+Enter");
  await expect(app.getByText("Page 2 of 2")).toBeVisible();
  await expect(app.getByLabel("Skipped answer")).toHaveCount(0);
  await app.getByLabel("Detailed answer").fill("Line one");
  await app.getByLabel("Detailed answer").press("End");
  await app.getByLabel("Detailed answer").press("Enter");
  await app.getByLabel("Detailed answer").press("a");
  await expect(app.getByLabel("Detailed answer")).toHaveValue("Line one\na");
  expect(
    await page.evaluate(() => (window as unknown as Harness).messages.length),
  ).toBe(0);
  await app.getByLabel("Detailed answer").press("Control+Enter");
  const submitted = await result(page);
  expect(submitted).toMatchObject({ first: "Ready", details: "Line one\na" });
  expect(submitted).not.toHaveProperty("skipped");
});

test("recovers focus when a revealed field disappears and fits narrow host windows", async ({
  page,
}) => {
  await page.setViewportSize({ width: 360, height: 720 });
  const app = await show(
    page,
    {
      title: "Focus and layout",
      elements: [
        {
          check: "Reveal",
          id: "reveal",
          default: true,
          reveals: [{ input: "Revealed answer", id: "answer" }],
        },
        {
          markdown:
            "A paragraph with **emphasis** and ![external media](https://example.com/image.png)",
        },
      ],
    },
    { theme: "light" },
  );
  await app.getByLabel("Revealed answer").fill("Retained");
  await app.getByLabel("Revealed answer").focus();
  await app
    .getByLabel("Reveal", { exact: true })
    .evaluate((element) => (element as HTMLInputElement).click());
  await expect(app.getByLabel("Reveal", { exact: true })).toBeFocused();
  await expect(app.locator("img")).toHaveCount(0);
  expect(
    await app
      .locator("html")
      .evaluate((element) => element.scrollWidth <= element.clientWidth),
  ).toBe(true);
  await app.getByLabel("Reveal", { exact: true }).check();
  await expect(app.getByLabel("Revealed answer")).toHaveValue("Retained");
});

test("host theme takes precedence over the operating system", async ({
  page,
}) => {
  await page.emulateMedia({ colorScheme: "dark" });
  let app = await show(
    page,
    { title: "Host appearance", elements: [{ input: "Name" }] },
    { theme: "light" },
  );
  expect(
    await app
      .locator("html")
      .evaluate((element) => getComputedStyle(element).colorScheme),
  ).toBe("light");
  await page.emulateMedia({ colorScheme: "light" });
  app = await show(
    page,
    { title: "Host appearance", elements: [{ input: "Name" }] },
    { theme: "dark" },
  );
  expect(
    await app
      .locator("html")
      .evaluate((element) => getComputedStyle(element).colorScheme),
  ).toBe("dark");
});

test("single-line Enter submits in the sandbox and IME Enter keeps editing", async ({
  page,
}) => {
  const app = await show(page, {
    title: "Keyboard submission",
    elements: [{ input: "Answer", id: "answer" }],
  });
  await app.getByLabel("Answer", { exact: true }).fill("A final answer");
  const prevented = await app
    .getByLabel("Answer", { exact: true })
    .evaluate((element) => {
      const event = new KeyboardEvent("keydown", {
        key: "Enter",
        ctrlKey: true,
        isComposing: true,
        bubbles: true,
        cancelable: true,
      });
      element.dispatchEvent(event);
      return event.defaultPrevented;
    });
  expect(prevented).toBe(false);
  expect(
    await page.evaluate(() => (window as unknown as Harness).messages.length),
  ).toBe(0);
  await app.getByLabel("Answer", { exact: true }).press("Enter");
  expect(await result(page)).toMatchObject({ answer: "A final answer" });
  await expect(
    app.getByRole("heading", { name: "Response complete" }),
  ).toBeVisible();
});

test("sliders preserve off-grid defaults and snap pointer and keyboard adjustments", async ({
  page,
}) => {
  const app = await show(page, {
    title: "Slider values",
    elements: [
      {
        slider: "Port",
        id: "port",
        min: 1000,
        max: 10000,
        step: 100,
        default: 8080,
      },
      {
        slider: "Untouched port",
        id: "untouched",
        min: 1000,
        max: 10000,
        step: 100,
        default: 8080,
      },
      { slider: "Endpoint", id: "endpoint", min: 0, max: 10, step: 3 },
      { slider: "Implicit midpoint", id: "midpoint", min: 0, max: 3 },
    ],
  });
  const port = app.getByRole("slider", { name: /^Port(?: |$)/ });
  const endpoint = app.getByRole("slider", { name: /^Endpoint(?: |$)/ });
  await expect(port).toHaveValue("8080");
  await expect(
    app.getByRole("slider", { name: /^Untouched port(?: |$)/ }),
  ).toHaveValue("8080");
  await expect(
    app.getByRole("slider", { name: /^Implicit midpoint(?: |$)/ }),
  ).toHaveValue("2");
  await port.press("ArrowRight");
  await expect(port).toHaveValue("8100");
  await port.press("ArrowDown");
  await expect(port).toHaveValue("8000");
  await port.press("PageUp");
  await expect(port).toHaveValue("9000");
  await port.press("PageDown");
  await expect(port).toHaveValue("8000");
  await endpoint.press("End");
  await expect(endpoint).toHaveValue("10");
  await endpoint.press("ArrowLeft");
  await expect(endpoint).toHaveValue("9");
  await endpoint.press("ArrowRight");
  await expect(endpoint).toHaveValue("10");
  await endpoint.press("Home");
  await expect(endpoint).toHaveValue("0");
  await endpoint.evaluate((element) => {
    const slider = element as HTMLInputElement;
    slider.value = "7.4";
    slider.dispatchEvent(new Event("input", { bubbles: true }));
  });
  await expect(endpoint).toHaveValue("6");
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  expect(await result(page)).toMatchObject({
    port: 8000,
    untouched: 8080,
    endpoint: 6,
    midpoint: 2,
  });
});

test("choice descriptions activate options and multi counts track selection", async ({
  page,
}) => {
  const app = await show(page, {
    title: "Readable choices",
    notes: false,
    elements: [
      {
        select: "Approach",
        options: [
          { value: "Repair", description: "Keep the current system" },
          { value: "Replace", description: "Start with a clean design" },
        ],
      },
      {
        multi: "Features",
        options: [
          { value: "Logs", description: "Record each operation" },
          { value: "Metrics", description: "Track performance over time" },
        ],
      },
      { slider: "Capacity", min: 5, max: 25, step: 5 },
    ],
  });
  await app.getByText("Keep the current system", { exact: true }).click();
  await expect(
    app.getByRole("radio", { name: "Repair", exact: true }),
  ).toBeChecked();
  await app.getByText("Start with a clean design", { exact: true }).click();
  await expect(
    app.getByRole("radio", { name: "Replace", exact: true }),
  ).toBeChecked();
  await expect(
    app.getByRole("radio", { name: "Repair", exact: true }),
  ).not.toBeChecked();
  await expect(app.locator(".field-multi .field-hint")).toContainText(
    "Choose any that apply",
  );
  await expect(app.getByText("0 selected", { exact: true })).toBeVisible();
  await app.getByText("Record each operation", { exact: true }).click();
  await expect(
    app.getByRole("checkbox", { name: "Logs", exact: true }),
  ).toBeChecked();
  await expect(app.getByText("1 selected", { exact: true })).toBeVisible();
  await app.getByText("Track performance over time", { exact: true }).click();
  await expect(app.getByText("2 selected", { exact: true })).toBeVisible();
  await app.getByText("Record each operation", { exact: true }).click();
  await expect(
    app.getByRole("checkbox", { name: "Logs", exact: true }),
  ).not.toBeChecked();
  await expect(app.getByText("1 selected", { exact: true })).toBeVisible();
  await expect(app.locator(".range-bounds span")).toHaveText(["5", "25"]);
  await expect(app.getByLabel(/Additional notes/)).toHaveCount(0);
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  expect(await result(page)).toMatchObject({
    approach: "Replace",
    features: ["Metrics"],
  });
});

test("notes are visibly optional and copied feedback corresponds to the response", async ({
  page,
}) => {
  const app = await show(page, {
    title: "Optional context",
    elements: [{ input: "Answer" }],
  });
  await expect(app.getByText("Optional", { exact: true })).toBeVisible();
  const notes = app.getByLabel(/Additional notes/);
  await expect(notes).toHaveAttribute(
    "placeholder",
    "Anything else, or anything we missed?",
  );
  await notes.fill("  An extra detail  ");
  await app.getByLabel("Answer", { exact: true }).fill("Ready");
  await app.locator("html").evaluate(() => {
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText: async (text: string) => {
          (window as unknown as { copiedResponse: string }).copiedResponse =
            text;
        },
      },
    });
  });
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  const submitted = await result(page);
  expect(submitted).toMatchObject({
    answer: "Ready",
    _notes: "An extra detail",
  });
  await app.getByRole("button", { name: "Copy response", exact: true }).click();
  await expect(
    app.getByRole("button", { name: "Copied", exact: true }),
  ).toBeVisible();
  const copied = await app
    .locator("html")
    .evaluate(
      () => (window as unknown as { copiedResponse: string }).copiedResponse,
    );
  expect(JSON.parse(copied)).toEqual(submitted);
});
