import { expect, test } from "@playwright/test";

test("packing follows container width, retains state and preserves reading order", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1040, height: 900 });
  await page.goto("/tests/host.html");
  await page.evaluate(async () => {
    await (
      window as unknown as { showForm(d: unknown): Promise<void> }
    ).showForm({
      notes: false,
      elements: [
        {
          group: "Execution",
          elements: [
            { slider: "Workers", id: "workers", min: 1, max: 16, default: 4 },
            { input: "Command", id: "command", default: "npm test" },
          ],
        },
        {
          group: "Evidence",
          elements: [
            { check: "Timings", id: "timings" },
            { check: "Costs", id: "costs" },
          ],
        },
        {
          multi: "Targets",
          id: "targets",
          options: ["Linux", "macOS", "Windows"],
        },
        {
          check: "Inspect details",
          id: "inspect",
          reveals: [{ input: "Detail", id: "detail" }],
        },
      ],
    });
  });
  const app = page.frameLocator('[data-testid="app"]');
  const groups = app.locator(".field-run-groups > .field");
  const first = await groups.nth(0).boundingBox();
  const second = await groups.nth(1).boundingBox();
  expect(Math.abs(first!.y - second!.y)).toBeLessThan(2);
  expect(second!.x).toBeGreaterThan(first!.x + first!.width);
  const linux = await app
    .getByRole("checkbox", { name: "Linux", exact: true })
    .boundingBox();
  const mac = await app
    .getByRole("checkbox", { name: "macOS", exact: true })
    .boundingBox();
  expect(Math.abs(linux!.y - mac!.y)).toBeLessThan(2);
  expect(mac!.x).toBeGreaterThan(linux!.x);
  await app.getByLabel("Command", { exact: true }).fill("npm run verify");
  await app.getByLabel("Command", { exact: true }).press("Tab");
  await expect(
    app.getByRole("checkbox", { name: "Timings", exact: true }),
  ).toBeFocused();
  await app.getByRole("checkbox", { name: "Linux", exact: true }).check();
  await app
    .getByRole("checkbox", { name: "Inspect details", exact: true })
    .check();
  await app.getByLabel("Detail", { exact: true }).fill("Keep this draft");
  await page.setViewportSize({ width: 360, height: 900 });
  await expect(app.getByLabel("Command", { exact: true })).toHaveValue(
    "npm run verify",
  );
  await expect(app.getByLabel("Detail", { exact: true })).toHaveValue(
    "Keep this draft",
  );
  const narrowFirst = await groups.nth(0).boundingBox();
  const narrowSecond = await groups.nth(1).boundingBox();
  expect(narrowSecond!.y).toBeGreaterThan(narrowFirst!.y + narrowFirst!.height);
  expect(
    await app
      .locator("main")
      .evaluate(
        () =>
          document.documentElement.scrollWidth >
          document.documentElement.clientWidth,
      ),
  ).toBe(false);
  await app
    .getByRole("checkbox", { name: "Inspect details", exact: true })
    .uncheck();
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  await expect(
    app.getByRole("heading", { name: "Response complete" }),
  ).toBeVisible();
  const result = await page.evaluate(
    () =>
      (window as unknown as { messages: { result: Record<string, unknown> }[] })
        .messages[0].result,
  );
  expect(result).toMatchObject({
    command: "npm run verify",
    targets: ["Linux"],
  });
  expect(result).not.toHaveProperty("detail");
});

test("branch columns preserve subtrees, corrections and focus across widths", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1040, height: 900 });
  await page.goto("/tests/host.html");
  await page.evaluate(async () => {
    await (
      window as unknown as { showForm(d: unknown): Promise<void> }
    ).showForm({
      notes: false,
      elements: [
        {
          multi: "Explore together",
          id: "paths",
          options: [
            {
              id: "measure",
              label: "Measure",
              children: [
                { input: "Measurement command", id: "command" },
                {
                  disclosure: "Supporting detail",
                  elements: [{ input: "Detail", id: "detail" }],
                },
              ],
            },
            {
              id: "compare",
              label:
                "Compare alternatives with a deliberately long explanation that must wrap without squeezing the controls",
              children: [{ slider: "Samples", id: "samples", min: 1, max: 20 }],
            },
            {
              id: "inspect",
              label: "Inspect",
              children: [{ check: "Include logs", id: "logs" }],
            },
          ],
        },
        {
          group: "Shared follow-up",
          when: "any(selected(paths, measure), selected(paths, compare))",
          elements: [{ input: "Outcome", id: "outcome" }],
        },
      ],
    });
  });
  const app = page.frameLocator('[data-testid="app"]');
  const options = app.locator(".choices-branches > .option");
  const measure = app.getByRole("checkbox", { name: "Measure", exact: true });
  const compare = app.getByRole("checkbox", { name: /^Compare alternatives/ });
  const original = await options.nth(1).boundingBox();
  await measure.check();
  await compare.check();
  await app
    .getByLabel("Measurement command", { exact: true })
    .fill("npm run measure");
  await app.locator("summary").filter({ hasText: "Supporting detail" }).click();
  await app.getByLabel("Detail", { exact: true }).fill("Preserve this");
  await app.getByLabel("Detail", { exact: true }).press("Tab");
  await expect(compare).toBeFocused();
  const left = await options.nth(0).boundingBox();
  const right = await options.nth(1).boundingBox();
  expect(right!.x).toBe(original!.x);
  expect(Math.abs(left!.y - right!.y)).toBeLessThan(2);
  expect(right!.x).toBeGreaterThan(left!.x + left!.width);
  const other = options.filter({
    has: app.getByRole("checkbox", {
      name: "Other (please specify)",
      exact: true,
    }),
  });
  expect((await other.boundingBox())!.width).toBeGreaterThan(left!.width * 2);
  await app
    .getByRole("checkbox", { name: "Other (please specify)", exact: true })
    .check();
  await app
    .getByLabel("Please specify", { exact: true })
    .fill("Also inspect scheduling");
  // The narrow branch must stack its label above its input even in a wide host.
  const label = await app
    .locator("label")
    .filter({ hasText: /^Measurement command$/ })
    .boundingBox();
  const input = await app
    .getByLabel("Measurement command", { exact: true })
    .boundingBox();
  expect(input!.y).toBeGreaterThanOrEqual(label!.y + label!.height);
  for (const width of [1440, 720, 360, 1040]) {
    await page.setViewportSize({ width, height: 900 });
    const resizedLabel = await app
      .locator("label")
      .filter({ hasText: /^Measurement command$/ })
      .boundingBox();
    const resizedInput = await app
      .getByLabel("Measurement command", { exact: true })
      .boundingBox();
    expect(resizedInput!.y).toBeGreaterThanOrEqual(
      resizedLabel!.y + resizedLabel!.height,
    );
    await expect(
      app.getByLabel("Measurement command", { exact: true }),
    ).toHaveValue("npm run measure");
    await expect(app.getByLabel("Detail", { exact: true })).toHaveValue(
      "Preserve this",
    );
    if (width <= 720) {
      const a = await options.nth(0).boundingBox();
      const b = await options.nth(1).boundingBox();
      expect(b!.y).toBeGreaterThan(a!.y + a!.height);
    }
    expect(
      await app
        .locator("main")
        .evaluate(
          () =>
            document.documentElement.scrollWidth >
            document.documentElement.clientWidth,
        ),
    ).toBe(false);
  }
  await measure.uncheck();
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  await expect(
    app.getByRole("heading", { name: "Response complete" }),
  ).toBeVisible();
  const result = await page.evaluate(
    () =>
      (window as unknown as { messages: { result: Record<string, unknown> }[] })
        .messages[0].result,
  );
  expect(result.paths).toEqual(["compare", "_other"]);
  expect(result.paths_other_text).toBe("Also inspect scheduling");
  expect(result).not.toHaveProperty("command");
  expect(result).not.toHaveProperty("detail");
  expect(result).toHaveProperty("outcome");
});
