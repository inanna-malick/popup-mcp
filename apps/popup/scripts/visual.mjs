import { chromium } from "@playwright/test";
import { spawn } from "node:child_process";
import { mkdir, writeFile } from "node:fs/promises";
import { resolve } from "node:path";
import { scenarios } from "../evals/visual/calls.mjs";

const output = resolve(process.argv[2] || ".visual/latest");
const chosen = scenarios.filter(
  (s) => !process.argv[3] || s.id === process.argv[3],
);
if (!chosen.length) throw new Error("Unknown scenario");
await mkdir(output, { recursive: true });
const port = 4175;
const base = `http://127.0.0.1:${port}`;
const server = spawn(
  process.execPath,
  [
    "node_modules/vite/bin/vite.js",
    "--host",
    "127.0.0.1",
    "--port",
    String(port),
    "--strictPort",
  ],
  { stdio: "pipe" },
);
let failure = "";
server.stderr.on("data", (data) => {
  failure += data;
});
let browser;
const manifest = {
  capturedAt: new Date().toISOString(),
  transport: process.env.POPUP_MCP_URL || "local official AppBridge harness",
  captures: [],
};
try {
  for (let attempt = 0; ; attempt++) {
    if (server.exitCode !== null)
      throw new Error(`Preview server exited: ${failure}`);
    try {
      if ((await fetch(`${base}/tests/host.html`)).ok) break;
    } catch {}
    if (attempt > 50)
      throw new Error(`Preview server did not start: ${failure}`);
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  browser = await chromium.launch();
  for (const scenario of chosen) {
    await writeFile(
      resolve(output, `${scenario.id}.call.json`),
      JSON.stringify(scenario.call, null, 2) + "\n",
    );
    for (const [variant, width, theme] of [
      ["light", 720, "light"],
      ["dark", 720, "dark"],
      ["wide", 1040, "light"],
      ["wide-dark", 1040, "dark"],
      ["large", 1440, "light"],
      ["zoom", 720, "light"],
      ["narrow", 360, "light"],
    ]) {
      const page = await browser.newPage({
        viewport: { width, height: 900 },
        // A 1440px display at 200% browser zoom has a 720 CSS-pixel viewport.
        deviceScaleFactor: variant === "zoom" ? 2 : 1,
        reducedMotion: "reduce",
        hasTouch: variant === "narrow",
      });
      page.setDefaultTimeout(10000);
      const errors = [];
      page.on("pageerror", (error) => errors.push(error.message));
      await page.goto(`${base}/tests/host.html`);
      await page.evaluate(
        async ({ definition, theme, endpoint, rejectMessages }) => {
          await Promise.race([
            window.showForm(definition, { theme, endpoint, rejectMessages }),
            new Promise((_, reject) =>
              setTimeout(
                () => reject(new Error("Host initialization timed out")),
                15000,
              ),
            ),
          ]);
        },
        {
          definition: scenario.call.arguments,
          theme,
          endpoint: process.env.POPUP_MCP_URL,
          rejectMessages: variant === "light" ? 1 : 0,
        },
      );
      const frame = page.frames().find((frame) => frame.parentFrame());
      await frame.locator("main.popup").waitFor();
      try {
        await frame
          .getByRole("button", {
            name: scenario.call.arguments.submit,
            exact: true,
          })
          .waitFor();
      } catch (error) {
        console.error(await frame.locator("body").innerText());
        throw error;
      }
      const capture = async (state) => {
        await page.mouse.move(width - 1, 0);
        const metrics = await frame.locator("main").evaluate((element) => ({
          height: Math.ceil(element.getBoundingClientRect().height),
          overflow:
            document.documentElement.scrollWidth >
            document.documentElement.clientWidth,
        }));
        await page.locator("iframe").evaluate((element, height) => {
          element.style.height = `${height}px`;
          element.style.display = "block";
        }, metrics.height);
        await page.setViewportSize({ width, height: metrics.height });
        const file = `${scenario.id}-${variant}-${state}.png`;
        await page.screenshot({
          path: resolve(output, file),
          fullPage: true,
          animations: "disabled",
        });
        manifest.captures.push({
          scenario: scenario.id,
          variant,
          state,
          file,
          width,
          ...metrics,
        });
        if (metrics.overflow) throw new Error(`Horizontal overflow: ${file}`);
      };
      await capture("initial");
      for (const action of scenario.explore) {
        if (action.summary)
          await frame
            .locator("summary")
            .filter({ hasText: action.summary })
            .click();
        else
          await frame
            .getByRole(action.role, { name: action.name, exact: true })
            .click();
      }
      await capture("expanded");
      if (variant === "light") {
        await frame
          .getByRole("button", {
            name: scenario.call.arguments.submit,
            exact: true,
          })
          .click();
        await frame
          .getByRole("heading", {
            name: "Your response could not be delivered",
            exact: true,
          })
          .waitFor();
        await capture("delivery-error");
        await frame
          .getByRole("button", { name: "Retry delivery", exact: true })
          .click();
        await frame
          .getByRole("heading", { name: "Response complete", exact: true })
          .waitFor();
        await capture("submitted");
        const messages = await page.evaluate(() => window.messages);
        await writeFile(
          resolve(output, `${scenario.id}.result.json`),
          JSON.stringify(messages, null, 2) + "\n",
        );
      }
      if (errors.length) throw new Error(errors.join("\n"));
      await page.close();
    }
  }
  await writeFile(
    resolve(output, "manifest.json"),
    JSON.stringify(manifest, null, 2) + "\n",
  );
  const escape = (text) =>
    String(text).replace(
      /[&<>"']/g,
      (character) =>
        ({
          "&": "&amp;",
          "<": "&lt;",
          ">": "&gt;",
          '"': "&quot;",
          "'": "&#39;",
        })[character],
    );
  await writeFile(
    resolve(output, "index.html"),
    `<!doctype html><html lang="en"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Popup visual review</title><style>
    body{margin:24px;background:#e8eae9;color:#24302c;font:14px/1.5 system-ui}h1{font-size:22px}nav{display:flex;gap:20px}section{margin:32px 0}.screens{display:flex;align-items:flex-start;gap:20px;overflow:auto}figure{margin:0;flex:0 0 360px}img{display:block;width:100%;border:1px solid #c6ceca}figcaption{padding:8px 0}a{color:inherit}
    </style><h1>Popup · authored calls / rendered states</h1><p>${escape(manifest.transport)} · ${escape(manifest.capturedAt)}</p><nav>${chosen.map((s) => `<a href="#${escape(s.id)}">${escape(s.id)}</a>`).join("")}</nav>${chosen
      .map(
        (s) =>
          `<section id="${escape(s.id)}"><h2>${escape(s.call.arguments.title)}</h2><p>${escape(s.context)}</p><p><a href="${escape(s.id)}.call.json">Tool call</a> · <a href="${escape(s.id)}.result.json">Submitted result</a></p><div class="screens">${manifest.captures
            .filter((c) => c.scenario === s.id)
            .map(
              (c) =>
                `<figure><figcaption>${escape(c.variant)} / ${escape(c.state)} · ${c.width} × ${c.height}</figcaption><a href="${escape(c.file)}"><img loading="lazy" src="${escape(c.file)}" alt="${escape(`${s.id}, ${c.variant}, ${c.state}`)}"></a></figure>`,
            )
            .join("")}</div></section>`,
      )
      .join("")}</html>`,
  );
  console.log(
    `Captured ${manifest.captures.length} screens and ${chosen.length} calls/results in ${output}`,
  );
} finally {
  await browser?.close();
  server.kill("SIGTERM");
}
