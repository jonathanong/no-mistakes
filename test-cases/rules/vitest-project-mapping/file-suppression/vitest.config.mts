import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    projects: [
      {
        name: "unit-a",
        include: ["src/a.test.ts", "src/shared.test.ts"],
      },
      {
        name: "unit-b",
        include: ["src/shared.test.ts"],
      },
    ],
  },
});
