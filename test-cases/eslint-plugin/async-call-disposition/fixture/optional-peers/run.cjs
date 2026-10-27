const assert = require("node:assert/strict");
const { readFileSync } = require("node:fs");
const { createRequire, Module } = require("node:module");
const { join } = require("node:path");
const { Linter } = require(process.argv[2]);
const pluginRequire = createRequire(require.resolve("eslint-plugin-no-mistakes"));
const hasTypeScript = process.argv[3] === "typescript";
const incompatible = process.argv[3] === "incompatible";
for (const name of ["typescript", "@typescript-eslint/parser"]) {
  if (incompatible || (name === "typescript" && hasTypeScript)) pluginRequire.resolve(name);
  else assert.throws(() => pluginRequire.resolve(name), { code: "MODULE_NOT_FOUND" });
}
const loads = { typescript: 0, "@typescript-eslint/parser": 0 };
const originalLoad = Module._load;
Module._load = function (name, ...args) {
  if (Object.hasOwn(loads, name)) loads[name]++;
  return originalLoad.call(this, name, ...args);
};
const plugin = require("eslint-plugin-no-mistakes");
assert.deepEqual(loads, { typescript: 0, "@typescript-eslint/parser": 0 });
const targets = [{ sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["enqueueEmail"] }];
function lint(name, awaitingConsumers, configuredTargets = targets) {
  const filename = join(__dirname, name);
  return new Linter({ cwd: __dirname }).verify(readFileSync(filename, "utf8"), {
    plugins: { "no-mistakes": plugin },
    rules: { "no-mistakes/async-call-disposition": ["error", { targets: configuredTargets, awaitingConsumers }] },
  }, { filename }).map(({ messageId, line }) => [line, messageId]);
}
assert.deepEqual(lint("local.js"), []);
assert.deepEqual(lint("configured.js", [{
  sourceSpecifierPatterns: ["./consumer.js"], calleeNamePatterns: ["consume"],
  callbackArgumentIndexes: [0],
}, {
  sourceSpecifierPatterns: ["./consumer.js"], calleeNamePatterns: ["consumeProperties"],
  callbackArgumentProperties: [{ argumentIndex: 0, propertyNames: ["enqueueEmail"] }],
}]), []);
assert.deepEqual(loads, { typescript: 0, "@typescript-eslint/parser": 0 });
assert.deepEqual(lint("imported.js"), hasTypeScript ? [] : [[4, "disposition"], [6, "disposition"], [8, "disposition"]]);
const missing = hasTypeScript ? "@typescript-eslint/parser" : "typescript";
assert.equal(loads[missing], 1, "missing peers must be memoized per context");
assert.deepEqual(lint("imported.js"), hasTypeScript ? [] : [[4, "disposition"], [6, "disposition"], [8, "disposition"]]);
assert.equal(loads[missing], 2, "a new rule context retries optional peer loading");

assert.deepEqual(lint("recall.js"), [3, 5, 7, 9, 12, 15].map((line) => [line, "disposition"]));
assert.deepEqual(lint("unsupported.js"), [[3, "disposition"]]);
assert.deepEqual(lint("unsupported-enum.js"), [[3, "disposition"]]);

// The packed consumer has TypeScript but no parser. Keep the complete imported
// chain and a dropped-callback control so a missing finding cannot pass silently.
assert.deepEqual(lint("multi-hop/caller.js", undefined, [{
  sourceSpecifierPatterns: ["@app/jobs"],
  calleeNamePatterns: ["enqueueBatch"],
}]), (hasTypeScript ? [13, 18, 23, 28] : [8, 13, 18, 23, 28]).map((line) => [line, "disposition"]));
