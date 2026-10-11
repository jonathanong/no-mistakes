const { defineConfig } = require("vitest/config");

module.exports = defineConfig({
  test: {
    globals: true,
    // Compiled API calls share the CLI lock across concurrent test files.
    testTimeout: process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node") ? 20_000 : 5_000,
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
        "packages/no-mistakes/scripts/wait-npm-tarball.js",
        "packages/no-mistakes/scripts/smoke-published-native.js",
        "packages/eslint-plugin-no-mistakes/src/**/*.js",
      ],
      reporter: ["text", "lcov"],
      thresholds: {
        statements: 98,
        branches: 98,
        functions: 98,
        lines: 98,
      },
    },
  },
});
