import { defineConfig } from "vitest/config";
export default defineConfig({
  test: { include: ["tests/worker.integration.ts"], testTimeout: 30000 },
});
