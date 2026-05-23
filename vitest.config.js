const { defineConfig } = require("vitest/config");

module.exports = defineConfig({
  test: {
    globals: true,
    include: [
      "packages/*/scripts/*.test.js",
      "packages/playwright-ast-coverage/scripts/*.test.js",
      "packages/queue-ast-hop/scripts/*.test.js",
      "packages/server-ast-routes/scripts/*.test.js",
      "packages/eslint-plugin-no-mistakes/test/**/*.test.mjs",
      "tests/js/**/*.test.js",
    ],
    coverage: {
      provider: "v8",
      include: [
        "packages/no-mistakes-core/lib/**/*.js",
        "packages/*/scripts/install.js",
        "packages/*/scripts/install/**/*.js",
        "packages/eslint-plugin-no-mistakes/src/**/*.js",
      ],
      reporter: ["text", "lcov"],
      thresholds: {
        statements: 98,
        branches: 94,
        functions: 99,
        lines: 99,
      },
    },
  },
});
