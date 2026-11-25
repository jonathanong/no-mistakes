import { defineConfig } from "vitest/config";

const settings = {
  test: {
    retry: 2,
    include: ["../tests/**/*.test.ts"],
    projects: [
      { test: { name: "approved", retry: 3 } },
      { test: { name: "unit", retry: 0 } },
    ],
  },
};

export default defineConfig(settings);
