import { defineConfig } from "vitest/config";

export default defineConfig({
  test: { include: ["evals/comparison/*.evaluation.ts"] },
});
