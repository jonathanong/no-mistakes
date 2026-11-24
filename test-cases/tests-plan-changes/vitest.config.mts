import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    projects: [
      { test: { name: "reader", include: ["tests/reader.test.ts"] } },
      { test: { name: "browser", include: ["tests/browser.test.ts"] } },
      { test: { name: "example", include: ["tests/example.test.ts"] } },
      { test: { name: "empty", include: ["tests/empty/*.test.ts"] } },
    ],
  },
});
