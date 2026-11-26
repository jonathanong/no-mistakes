import { defineConfig } from "vitest/config";

declare const runtimeList: string[];

export default defineConfig({
  test: {
    exclude: ["**/node_modules/**", "**/.git/**", ...runtimeList],
  },
});
