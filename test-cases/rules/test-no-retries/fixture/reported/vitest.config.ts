import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    retry: 2,
    projects: [
      { test: { name: "smoke", retry: { count: 2 } } },
      { test: { name: "unit", retry: { count: 1 } } },
    ],
  },
});
