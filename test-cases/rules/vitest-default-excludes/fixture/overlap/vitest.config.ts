import { configDefaults, defineConfig, defineProject } from "vitest/config";

export default defineConfig({
  test: {
    exclude: ["**/node_modules/**"],
    projects: [
      defineProject({
        test: {
          exclude: ["**/dist/**"],
        },
      }),
      {
        test: {
          exclude: ["**/.git/**"],
        },
      },
      defineProject({
        test: {
          exclude: [...configDefaults.exclude, "**/tmp/**"],
        },
      }),
    ],
  },
});
