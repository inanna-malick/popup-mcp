import { expect, test } from "@playwright/test";

test("numeric drafts validate, normalize, cancel and stay local when hidden", async ({
  page,
}) => {
  await page.goto("/tests/host.html");
  await page.evaluate(async () => {
    await (window as any).showForm({
      notes: false,
      elements: [
        {
          check: "Tune",
          id: "tune",
          default: true,
          reveals: [
            {
              slider: "Budget",
              id: "budget",
              min: 0,
              max: 10,
              step: 0.25,
              default: 1.13,
            },
          ],
        },
      ],
    });
  });
  const app = page.frameLocator('[data-testid="app"]');
  const number = app.getByRole("spinbutton", { name: "Budget value" });
  const slider = app.getByRole("slider", { name: "Budget", exact: true });
  await expect(number).toHaveValue("1.13");
  await number.fill("2.38");
  await number.press("Enter");
  await expect(number).toHaveValue("2.5");
  await expect(slider).toHaveValue("2.5");
  await number.press("ArrowUp");
  await expect(number).toHaveValue("2.75");
  await number.press("ArrowDown");
  await expect(number).toHaveValue("2.5");
  await expect(
    app.getByRole("button", { name: "Submit", exact: true }),
  ).toBeVisible();
  await number.fill("99");
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  await expect(number).toBeFocused();
  await expect(number).toHaveAttribute("aria-invalid", "true");
  await expect(slider).toHaveValue("2.5");
  await number.press("Escape");
  await expect(number).toHaveValue("2.5");
  await number.fill("");
  await number.press("Enter");
  await expect(number).toHaveAccessibleDescription(
    "Enter a number from 0 to 10.",
  );
  await slider.press("ArrowRight");
  await expect(number).toHaveValue("2.75");
  await expect(number).toHaveAttribute("aria-invalid", "false");
  await number.fill("-1");
  await app.getByRole("checkbox", { name: "Tune", exact: true }).uncheck();
  await app.getByRole("checkbox", { name: "Tune", exact: true }).check();
  await expect(number).toHaveValue("-1");
  await app.getByRole("checkbox", { name: "Tune", exact: true }).uncheck();
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  await expect(
    app.getByRole("heading", { name: "Response complete" }),
  ).toBeVisible();
  const result = await page.evaluate(() => (window as any).messages[0].result);
  expect(result).not.toHaveProperty("budget");
});

test("ranking moves retain focus and submit visual order", async ({ page }) => {
  await page.goto("/tests/host.html");
  await page.evaluate(async () => {
    await (window as any).showForm({
      notes: false,
      elements: [
        {
          rank: "Priorities",
          id: "priorities",
          options: [
            { id: "safe", label: "Safety" },
            { id: "fast", label: "Speed" },
            { id: "cheap", label: "Cost" },
          ],
        },
      ],
    });
  });
  const app = page.frameLocator('[data-testid="app"]');
  const safety = app.getByRole("button", { name: "Safety", exact: true });
  const speed = app.getByRole("button", { name: "Speed", exact: true });
  await safety.click();
  await speed.click();
  await app.getByRole("button", { name: "Move Speed up", exact: true }).click();
  await expect(speed).toBeFocused();
  await expect(speed).toHaveAccessibleDescription("Priority 1 of 2");
  await expect(app.locator(".rank-row > .rank")).toHaveText([
    "1Speed",
    "2Safety",
    "+Cost",
  ]);
  await speed.press("Alt+ArrowDown");
  await expect(speed).toBeFocused();
  await expect(speed).toHaveAccessibleDescription("Priority 2 of 2");
  await expect(
    app.getByRole("button", { name: "Move Speed down" }),
  ).toBeDisabled();
  await app.getByRole("button", { name: "Cost", exact: true }).click();
  await safety.click();
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  await expect(
    app.getByRole("heading", { name: "Response complete" }),
  ).toBeVisible();
  expect(
    await page.evaluate(() => (window as any).messages[0].result.priorities),
  ).toEqual(["fast", "cheap"]);
});

test("numeric validation opens closed detail and prevents advancing pages", async ({
  page,
}) => {
  await page.goto("/tests/host.html");
  await page.evaluate(async () => {
    await (window as any).showForm({
      notes: false,
      elements: [
        {
          page: "Settings",
          elements: [
            {
              disclosure: "Fine tuning",
              open: true,
              elements: [
                {
                  slider: "Workers",
                  id: "workers",
                  min: 1,
                  max: 8,
                  default: 2,
                },
              ],
            },
          ],
        },
        { page: "Review", elements: [{ input: "Note", id: "note" }] },
      ],
    });
  });
  const app = page.frameLocator('[data-testid="app"]');
  const number = app.getByRole("spinbutton", { name: "Workers value" });
  await number.fill("99");
  await app.locator("summary").filter({ hasText: "Fine tuning" }).click();
  await app.getByRole("button", { name: "Next", exact: true }).click();
  await expect(number).toBeVisible();
  await expect(number).toBeFocused();
  await expect(app.getByText("Page 1 of 2", { exact: true })).toBeVisible();
  await number.fill("4");
  await app.getByRole("button", { name: "Next", exact: true }).click();
  await expect(app.getByText("Page 2 of 2", { exact: true })).toBeVisible();
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  await expect(
    app.getByRole("heading", { name: "Response complete" }),
  ).toBeVisible();
  expect(
    await page.evaluate(() => (window as any).messages[0].result.workers),
  ).toBe(4);
});

test("clipboard failure remains visible after delivery failure", async ({
  page,
}) => {
  await page.goto("/tests/host.html");
  await page.evaluate(async () => {
    await (window as any).showForm(
      {
        notes: false,
        elements: [
          { input: "Proposal", id: "proposal", default: "Keep this answer" },
        ],
      },
      { rejectMessages: 1 },
    );
  });
  const app = page.frameLocator('[data-testid="app"]');
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  await expect(
    app.getByRole("heading", { name: "Your response could not be delivered" }),
  ).toBeVisible();
  await app.locator("main").evaluate(() => {
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText: async () => {
          throw new Error("Denied");
        },
      },
    });
  });
  await app.getByRole("button", { name: "Copy response", exact: true }).click();
  await expect(
    app.getByText(
      "Clipboard unavailable. Expand Response data to select and copy your response.",
      { exact: true },
    ),
  ).toBeVisible();
  await app.getByText("Response data", { exact: true }).click();
  await expect(
    app.getByRole("textbox", { name: "Submitted response data" }),
  ).toHaveValue(/Keep this answer/);
});
