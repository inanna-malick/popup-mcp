import { expect, test } from "@playwright/test";
import type { Envelope } from "../src/bridge/delivery";

type Harness = {
  showForm(definition: unknown, options?: { endpoint?: string }): Promise<void>;
  messages: Envelope[];
};

test("radio choices clear reversibly while hidden drafts stay out of submission", async ({
  page,
}) => {
  await page.goto("/tests/host.html");
  await page.evaluate(async (endpoint) => {
    await (window as unknown as Harness).showForm(
      {
        title: "Explore a repair",
        notes: false,
        elements: [
          {
            select: "Approach",
            id: "approach",
            options: ["Repair", "Replace"],
            Repair: [{ input: "Repair plan", id: "plan" }],
          },
        ],
      },
      { endpoint },
    );
  }, process.env.POPUP_MCP_URL);
  const app = page.frameLocator('[data-testid="app"]');
  const repair = app.getByRole("radio", { name: "Repair", exact: true });
  const clear = app.getByRole("button", {
    name: "Clear Approach",
    exact: true,
  });
  await expect(clear).toHaveCount(0);
  await repair.check();
  await app
    .getByLabel("Repair plan", { exact: true })
    .fill("Preserve the working parts");
  await clear.click();
  await expect(repair).not.toBeChecked();
  await expect(repair).toBeFocused();
  await expect(app.getByLabel("Repair plan", { exact: true })).toHaveCount(0);
  await expect(clear).toHaveCount(0);
  await repair.check();
  await expect(app.getByLabel("Repair plan", { exact: true })).toHaveValue(
    "Preserve the working parts",
  );
  await clear.click();
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  await expect
    .poll(() =>
      page.evaluate(() => (window as unknown as Harness).messages.length),
    )
    .toBe(1);
  const submitted = await page.evaluate(
    () => (window as unknown as Harness).messages[0].result,
  );
  expect(submitted).toMatchObject({ approach: null });
  expect(submitted).not.toHaveProperty("plan");
});

for (const theme of ["light", "dark"] as const) {
  test(`${theme} controls and placeholders retain readable contrast`, async ({
    page,
  }) => {
    await page.goto("/tests/host.html");
    await page.evaluate(async (theme) => {
      await (
        window as unknown as { showForm(d: unknown, o: unknown): Promise<void> }
      ).showForm(
        {
          elements: [
            { check: "Include details", id: "details" },
            { input: "Command", id: "command" },
          ],
        },
        { theme },
      );
    }, theme);
    const app = page.frameLocator('[data-testid="app"]');
    const ratios = await app.locator("main").evaluate((main) => {
      const rgb = (color: string) => {
        const canvas = document.createElement("canvas");
        canvas.width = canvas.height = 1;
        const context = canvas.getContext("2d")!;
        context.fillStyle = color;
        context.fillRect(0, 0, 1, 1);
        return Array.from(context.getImageData(0, 0, 1, 1).data)
          .slice(0, 3)
          .map((v) => {
            const s = v / 255;
            return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
          });
      };
      const luminance = (color: string) => {
        const [r, g, b] = rgb(color);
        return 0.2126 * r + 0.7152 * g + 0.0722 * b;
      };
      const contrast = (a: string, b: string) => {
        const x = luminance(a),
          y = luminance(b);
        return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
      };
      const check = getComputedStyle(
        main.querySelector('input[type="checkbox"]')!,
      );
      const input = getComputedStyle(main.querySelector(".field-input input")!);
      const notes = main.querySelector("textarea")!;
      const placeholder = getComputedStyle(notes, "::placeholder");
      const palette = getComputedStyle(document.documentElement);
      const token = (name: string) =>
        palette.getPropertyValue(`--popup-${name}`).trim();
      const pairs: [string, string, number][] = [
        ["text", "background", 4.5],
        ["text", "raised", 4.5],
        ["muted", "surface", 4.5],
        ["muted", "raised", 4.5],
        ["accent", "raised", 4.5],
        ["accent-ink", "accent", 4.5],
        ["accent-ink", "accent-hover", 4.5],
        ["selection-ink", "selection", 4.5],
        ["focus", "background", 3],
        ["focus", "raised", 3],
        ["control-line", "surface", 3],
        ["control-line", "raised", 3],
        ["error", "error-surface", 4.5],
        ["error", "background", 4.5],
        ["error-border", "error-surface", 3],
        ["success", "background", 4.5],
        ["success", "success-surface", 4.5],
      ];
      return {
        failedPairs: pairs.filter(
          ([a, b, min]) => contrast(token(a), token(b)) < min,
        ),
        check: contrast(check.borderTopColor, check.backgroundColor),
        input: contrast(input.borderTopColor, input.backgroundColor),
        placeholder: contrast(
          placeholder.color,
          getComputedStyle(notes).backgroundColor,
        ),
        opacity: placeholder.opacity,
      };
    });
    expect(ratios.check).toBeGreaterThanOrEqual(3);
    expect(ratios.failedPairs).toEqual([]);
    expect(ratios.input).toBeGreaterThanOrEqual(3);
    expect(ratios.placeholder).toBeGreaterThanOrEqual(4.5);
    expect(ratios.opacity).toBe("1");
  });
}

test("ranked choices expose ordinal positions and announce the resulting order", async ({
  page,
}) => {
  await page.goto("/tests/host.html");
  await page.evaluate(async () => {
    await (
      window as unknown as { showForm(d: unknown): Promise<void> }
    ).showForm({
      notes: false,
      elements: [
        {
          rank: "Priorities",
          id: "priorities",
          options: ["Safety", "Speed", "Cost"],
        },
      ],
    });
  });
  const app = page.frameLocator('[data-testid="app"]');
  const safety = app.getByRole("button", { name: "Safety", exact: true });
  const speed = app.getByRole("button", { name: "Speed", exact: true });
  await expect(safety).toHaveAccessibleDescription("Not ranked");
  await safety.click();
  await speed.click();
  await expect(safety).toHaveAccessibleDescription("Priority 1 of 2");
  await expect(speed).toHaveAccessibleDescription("Priority 2 of 2");
  await safety.click();
  await expect(speed).toHaveAccessibleDescription("Priority 1 of 1");
  await expect(app.locator(".rank-status")).toHaveText(
    "Priority order: 1. Speed",
  );
  await speed.click();
  await expect(app.locator(".rank-status")).toHaveText(
    "No priorities selected",
  );
});
