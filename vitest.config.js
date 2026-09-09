const { defineConfig } = require("vitest/config");

module.exports = defineConfig({
  test: {
    globals: true,
    setupFiles: ["tests/js/setup.js"],
    include: [
      "packages/*/scripts/*.test.js",
      "packages/eslint-plugin-no-mistakes/test/**/*.test.mjs",
      "tests/js/**/*.test.js",
    ],
    coverage: {
      provider: "v8",
      include: [
        "packages/no-mistakes/planning-impact-artifacts.js",
        "packages/no-mistakes/planning-impact-artifacts-files.js",
        "packages/no-mistakes/planning-impact-artifacts-inputs.js",
        "packages/no-mistakes/planning-impact-cli.js",
        "packages/no-mistakes/bin/no-mistakes.js",
        "packages/no-mistakes/scripts/native-package.js",
        "packages/eslint-plugin-no-mistakes/src/**/*.js",
      ],
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
