const { defineConfig } = require("vitest/config");

module.exports = defineConfig({
  test: {
    coverage: {
      provider: "v8",
      include: ["src/**/*.js"],
      reporter: ["text", "lcov"],
      thresholds: {
        statements: 95,
        branches: 94,
        functions: 95,
        lines: 95,
      },
    },
  },
});
