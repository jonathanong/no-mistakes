const { defineConfig } = require("vitest/config");

module.exports = defineConfig({
  test: {
    coverage: {
      provider: "v8",
      include: ["src/**/*.js"],
      reporter: ["text", "lcov"],
      thresholds: {
        statements: 98,
        branches: 95,
        functions: 99,
        lines: 99,
      },
    },
  },
});
