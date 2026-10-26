import { expect, test } from "@playwright/test";
import type { Envelope } from "../src/bridge/delivery";
type Host = {
  showForm(
    definition: unknown,
    options?: Record<string, unknown>,
  ): Promise<void>;
  messages: Envelope[];
  attempts: Envelope[];
  bridge: { sendToolResult(result: unknown): Promise<void> };
};
const form = {
  title: "A useful next step",
  elements: [
    {
      select: "Approach",
      id: "approach",
      options: ["Investigate", "Fix"],
      Investigate: [{ input: "Question", id: "question" }],
    },
  ],
};
test("Worker preserves stable identities, disclosure answers and shared follow-ups", async ({
  page,
}) => {
  test.skip(
    !process.env.POPUP_MCP_URL,
    "Set POPUP_MCP_URL for real Worker verification",
  );
  const definition = {
    notes: false,
    elements: [
      {
        multi: "Investigations",
        id: "inspections",
        options: [
          {
            id: "branch",
            label: "Branch behavior",
            recommendation: "Check the reported issue",
            children: [{ check: "Reopen branches", id: "reopen" }],
          },
          {
            id: "values",
            label: "Submitted values",
            children: [
              {
                disclosure: "Inspect expected result",
                elements: [
                  {
                    input: "Expected observation",
                    id: "observation",
                    default: "Active answers only",
                  },
                ],
              },
            ],
          },
        ],
      },
      {
        check: "Record evidence",
        id: "record",
        when: "selected(inspections, branch) || selected(inspections, values)",
      },
    ],
  };
  await page.goto("/tests/host.html");
  await page.evaluate(
    async ({ definition, endpoint }) =>
      (window as unknown as Host).showForm(definition, { endpoint }),
    { definition, endpoint: process.env.POPUP_MCP_URL },
  );
  const app = page.frameLocator('[data-testid="app"]');
  await expect(
    app.getByRole("checkbox", { name: "Branch behavior", exact: true }),
  ).not.toBeChecked();
  await expect(
    app.getByText("Check the reported issue", { exact: false }),
  ).toBeVisible();
  await app
    .getByRole("checkbox", { name: "Branch behavior", exact: true })
    .check();
  await app
    .getByRole("checkbox", { name: "Submitted values", exact: true })
    .check();
  await expect(
    app.getByRole("checkbox", { name: "Record evidence", exact: true }),
  ).toHaveCount(1);
  await app
    .getByRole("checkbox", { name: "Record evidence", exact: true })
    .check();
  await app
    .getByRole("checkbox", { name: "Reopen branches", exact: true })
    .check();
  await app
    .getByRole("checkbox", { name: "Branch behavior", exact: true })
    .uncheck();
  await expect(
    app.getByLabel("Expected observation", { exact: true }),
  ).toBeHidden();
  await app
    .getByRole("checkbox", { name: "Other (please specify)", exact: true })
    .check();
  await app
    .getByLabel("Please specify", { exact: true })
    .fill("Include keyboard behavior");
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  await expect(
    app.getByRole("heading", { name: "Response complete" }),
  ).toBeVisible();
  const responses = await page.evaluate(
    () => (window as unknown as Host).messages,
  );
  expect(responses).toHaveLength(1);
  expect(responses[0].result).toMatchObject({
    status: "completed",
    inspections: ["values", "_other"],
    observation: "Active answers only",
    record: true,
    inspections_other_text: "Include keyboard behavior",
    _option_labels: {
      inspections: {
        values: "Submitted values",
        _other: "Other (please specify)",
      },
    },
  });
  expect(responses[0].result).not.toHaveProperty("reopen");
  await expect(app.locator("dl.summary")).toContainText(
    "Submitted values, Other (please specify)",
  );
});
test("unsupported host messaging preserves a copyable answer", async ({
  page,
}) => {
  await page.goto("/tests/host.html");
  await page.evaluate(
    (definition) =>
      (window as unknown as Host).showForm(definition, { messaging: false }),
    form,
  );
  const app = page.frameLocator('[data-testid="app"]');
  await app.getByRole("radio", { name: "Fix", exact: true }).check();
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  await app.getByText("Delivery details", { exact: true }).click();
  await expect(app.getByText(/host cannot receive/)).toBeVisible();
  await app.getByText("Response data", { exact: true }).click();
  await expect(
    app.getByRole("textbox", { name: "Submitted response data" }),
  ).toHaveValue(/"approach": "Fix"/);
  expect(
    await page.evaluate(() => (window as unknown as Host).messages),
  ).toHaveLength(0);
});
test("replayed results do not reset drafts and successful receipts prevent resubmission", async ({
  page,
}) => {
  await page.goto("/tests/host.html");
  await page.evaluate(
    (definition) =>
      (window as unknown as Host).showForm(definition, {
        formId: "receipt-test",
      }),
    form,
  );
  const app = page.frameLocator('[data-testid="app"]');
  await app.getByRole("radio", { name: "Investigate", exact: true }).check();
  await app.getByLabel("Question", { exact: true }).fill("What changed?");
  await page.evaluate(
    (definition) =>
      (window as unknown as Host).bridge.sendToolResult({
        content: [],
        structuredContent: {
          status: "awaiting_input",
          formId: "receipt-test",
          definition,
        },
      }),
    form,
  );
  await expect(app.getByLabel("Question", { exact: true })).toHaveValue(
    "What changed?",
  );
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  await expect(
    app.getByRole("heading", { name: "Response complete" }),
  ).toBeVisible();
  await page.evaluate(
    (definition) =>
      (window as unknown as Host).showForm(definition, {
        formId: "receipt-test",
      }),
    form,
  );
  await expect(
    app.getByRole("heading", { name: "Response complete" }),
  ).toBeVisible();
  await expect(
    app.getByRole("button", { name: "Submit", exact: true }),
  ).toHaveCount(0);
});
test("Worker HTTP tool → served iframe → user response through official bridge", async ({
  page,
}) => {
  test.skip(
    !process.env.POPUP_MCP_URL,
    "Set POPUP_MCP_URL for real Worker verification",
  );
  await page.goto("/tests/host.html");
  await page.evaluate(
    async ({ definition, endpoint }) =>
      (window as unknown as Host).showForm(definition, { endpoint }),
    { definition: form, endpoint: process.env.POPUP_MCP_URL },
  );
  const app = page.frameLocator('[data-testid="app"]');
  await app
    .getByRole("radio", { name: "Other (please specify)", exact: true })
    .check();
  await app
    .getByLabel("Please specify", { exact: true })
    .fill("Pause and explain the tradeoffs");
  await app.getByRole("button", { name: "Submit", exact: true }).click();
  await expect(
    app.getByRole("heading", { name: "Response complete" }),
  ).toBeVisible();
  const responses = await page.evaluate(
    () => (window as unknown as Host).messages,
  );
  expect(responses).toHaveLength(1);
  expect(responses[0].result).toMatchObject({
    status: "completed",
    approach_other_text: "Pause and explain the tradeoffs",
  });
});
