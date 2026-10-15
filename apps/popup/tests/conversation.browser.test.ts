import { expect, test } from "@playwright/test";
import type { Envelope } from "../src/bridge/delivery";

type Host = {
  showForm(
    definition: unknown,
    options?: Record<string, unknown>,
  ): Promise<void>;
  messages: Envelope[];
  sizes: { height?: number; width?: number }[];
};

test("titleless conversational branches compose, resize, and preserve only active answers", async ({
  page,
}) => {
  await page.setViewportSize({ width: 360, height: 800 });
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/tests/host.html");
  await page.evaluate(() =>
    (window as unknown as Host).showForm({
      elements: [
        {
          multi: "Where should we go next?",
          id: "directions",
          options: ["Caching", "Parallelism"],
          Caching: [
            {
              check: "Cache build outputs",
              id: "outputs",
              reveals: [
                {
                  select: "Invalidation",
                  id: "invalidation",
                  options: ["Explicit inputs", "Investigate first"],
                  "Explicit inputs": [
                    {
                      input: "Cache key proposal",
                      id: "key",
                      default: "OS + lockfile + build inputs",
                    },
                  ],
                },
              ],
            },
          ],
          Parallelism: [
            {
              input: "Isolation proposal",
              id: "isolation",
              default: "One database per job",
            },
          ],
        },
        {
          text: "Both approaches can share a read-only dependency cache.",
          when: "selected(directions, Caching) && selected(directions, Parallelism)",
        },
      ],
    }),
  );
  const app = page.frameLocator('[data-testid="app"]');
  await expect(app.locator(".brand, .form-header")).toHaveCount(0);
  await expect(app.getByRole("heading", { level: 1 })).toHaveCount(0);
  await expect(
    app.getByRole("checkbox", { name: "Caching", exact: true }),
  ).toBeFocused();
  await expect
    .poll(() => page.evaluate(() => (window as unknown as Host).sizes.length))
    .toBeGreaterThan(0);
  const initialHeight = await page.evaluate(
    () => (window as unknown as Host).sizes.at(-1)?.height ?? 0,
  );
  await app.getByRole("checkbox", { name: "Caching", exact: true }).check();
  await app.getByRole("checkbox", { name: "Parallelism", exact: true }).check();
  await app
    .getByRole("checkbox", { name: "Cache build outputs", exact: true })
    .check();
  await app
    .getByRole("radio", { name: "Explicit inputs", exact: true })
    .check();
  await app
    .getByLabel("Cache key proposal")
    .fill("OS + Node + lockfile + source hash");
  await expect(app.getByLabel("Isolation proposal")).toBeVisible();
  await expect(
    app.getByText("Both approaches can share a read-only dependency cache."),
  ).toBeVisible();
  await expect
    .poll(() =>
      page.evaluate(
        () => (window as unknown as Host).sizes.at(-1)?.height ?? 0,
      ),
    )
    .toBeGreaterThan(initialHeight);
  const geometry = await app
    .getByLabel("Cache key proposal")
    .evaluate((element) => ({
      width: element.getBoundingClientRect().width,
      overflow:
        document.documentElement.scrollWidth >
        document.documentElement.clientWidth,
      animation: getComputedStyle(element.closest(".field")!).animationName,
    }));
  expect(geometry.width).toBeGreaterThanOrEqual(280);
  expect(geometry.overflow).toBe(false);
  expect(geometry.animation).toBe("none");
  await app.getByRole("checkbox", { name: "Caching", exact: true }).uncheck();
  await expect(app.getByLabel("Cache key proposal")).toHaveCount(0);
  await app.getByRole("checkbox", { name: "Caching", exact: true }).check();
  await expect(app.getByLabel("Cache key proposal")).toHaveValue(
    "OS + Node + lockfile + source hash",
  );
  await app.getByRole("checkbox", { name: "Caching", exact: true }).uncheck();
  await app
    .getByRole("checkbox", { name: "Other (please specify)", exact: true })
    .check();
  await app
    .getByLabel("Please specify", { exact: true })
    .fill("Measure the slowest step first");
  await app.getByLabel(/Additional notes/).fill("No coverage reductions");
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  await expect
    .poll(() =>
      page.evaluate(() => (window as unknown as Host).messages.length),
    )
    .toBe(1);
  const result = await page.evaluate(
    () => (window as unknown as Host).messages[0].result,
  );
  expect(result).toMatchObject({
    directions: ["Parallelism", "Other (please specify)"],
    isolation: "One database per job",
    directions_other_text: "Measure the slowest step first",
    _notes: "No coverage reductions",
  });
  for (const key of ["outputs", "invalidation", "key"])
    expect(result).not.toHaveProperty(key);
});

test("titleless pages move focus and host fonts reach the interaction without replacing the palette", async ({
  page,
}) => {
  await page.goto("/tests/host.html");
  await page.evaluate(() =>
    (window as unknown as Host).showForm(
      {
        title: "",
        notes: false,
        elements: [
          { page: "First", elements: [{ input: "First proposal" }] },
          { page: "Second", elements: [{ input: "Second proposal" }] },
        ],
      },
      {
        styleVariables: {
          "--font-sans": "monospace",
          "--color-background-primary": "#eef3f8",
          "--color-text-primary": "#123456",
        },
      },
    ),
  );
  const app = page.frameLocator('[data-testid="app"]');
  await expect(app.getByLabel("First proposal")).toBeFocused();
  await app.getByRole("button", { name: "Next", exact: true }).click();
  await expect(app.getByLabel("Second proposal")).toBeFocused();
  await app.getByRole("button", { name: "Back", exact: true }).click();
  await expect(app.getByLabel("First proposal")).toBeFocused();
  const style = await app.locator(".popup").evaluate((element) => ({
    font: getComputedStyle(element).fontFamily,
    color: getComputedStyle(element).color,
  }));
  expect(style.font).toContain("monospace");
  expect(style.color).toBe("rgb(3, 38, 42)");
});
