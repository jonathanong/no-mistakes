import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    projects: [
      {
        test: {
          name: "database",
          include: ["src/db/db.test.ts", "src/shared.test.ts"],
        },
      },
      {
        test: {
          name: "web",
          include: ["src/web/web.test.ts", "src/shared.test.ts"],
        },
      },
    ],
  },
});
