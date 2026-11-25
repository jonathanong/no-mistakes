import * as vitest from "vitest/config";

const inline = vitest.defineProject({ test: { ["retry"]: +2 } });
export default vitest.defineConfig({
  test: {
    retry: 5,
    ["retry"]: 0,
    projects: [
      "./opaque.config.ts",
      inline,
      { test: {} },
      ...[],
      injectedProject({ test: { retry: 99 } }),
      vitest.defineProject(),
      vitest.defineProject(...runtimeProjects),
    ],
  },
});
