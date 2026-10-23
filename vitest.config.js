const { defineConfig } = require("vitest/config");
const compiledNative = process.env.NO_MISTAKES_TEST_NAPI_ADDON_PATH?.endsWith(".node");

module.exports = defineConfig({
  test: {
    globals: true,
    // Native environment access must see each compiled worker's private runtime directory.
    pool: "forks",
    // Linux setup gives native forks distinct locks; other platforms share one.
    fileParallelism: !compiledNative || process.platform === "linux",
    testTimeout: compiledNative ? 20_000 : 5_000,
    setupFiles: ["tests/js/setup.js"],
    include: [
      "packages/*/scripts/*.test.js",
      "packages/eslint-plugin-no-mistakes/test/**/*.test.mjs",
      "tests/js/**/*.test.js",
      "packages/no-mistakes/test-timeout-policy/*.test.mjs",
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
        "packages/no-mistakes/test-timeout-policy/*.mjs",
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
