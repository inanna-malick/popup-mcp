import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import { viteSingleFile } from "vite-plugin-singlefile";
export default defineConfig({
  plugins: [solid(), viteSingleFile()],
  build: { outDir: "dist/ui", target: "es2022", sourcemap: false },
});
