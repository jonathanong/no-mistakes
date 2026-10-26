const assert = require("node:assert/strict");
const { createRequire } = require("node:module");
const { resolve } = require("node:path");
// Audit the actual Oxlint process, not Vitest's parent resolver or NODE_PATH.
const pluginRequire = createRequire(resolve(__dirname, "node_modules/eslint-plugin-no-mistakes/package.json"));
for (const name of ["typescript", "@typescript-eslint/parser"]) {
  if (process.env.NO_MISTAKES_TEST_PEERS === "present") assert.ok(pluginRequire.resolve(name));
  else assert.throws(() => pluginRequire.resolve(name), { code: "MODULE_NOT_FOUND" });
}
