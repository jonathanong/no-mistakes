import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { Linter } from "eslint";
import tsParser from "@typescript-eslint/parser";
import { it } from "vitest";
import { __dirname, plugin } from "./helpers.mjs";

const root = resolve(
  __dirname,
  "../../../test-cases/eslint-plugin/async-call-disposition/fixture/proven-consumers",
);
const targets = [{ sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["enqueueEmail"] }];
const propertyConsumer = {
  sourceSpecifierPatterns: ["@app/consumer"],
  calleeNamePatterns: ["consume"],
  callbackArgumentProperties: [
    { argumentIndex: 0, propertyNames: ["enqueueEmail", "enqueueEmailCallback"] },
  ],
};
function reports(name, typed = false, awaitingConsumers, parser = tsParser) {
  const filename = resolve(root, name);
  return new Linter({ cwd: dirname(filename) })
    .verify(
      readFileSync(filename, "utf8"),
      {
        files: ["**/*.ts"],
        languageOptions: {
          parser,
          parserOptions: typed
            ? { project: resolve(root, "tsconfig.json"), disallowAutomaticSingleRunInference: true }
            : {},
        },
        plugins: { "no-mistakes": plugin },
        rules: { "no-mistakes/async-call-disposition": ["error", { targets, awaitingConsumers }] },
      },
      { filename },
    )
    .map(({ line, messageId }) => [line, messageId]);
}
it.each([false, true])("proves same-file awaited callback invocations (typed=%s)", (typed) => {
  assert.deepEqual(
    reports("local.ts", typed),
    [16, 19, 25, 27, 31, 38, 43, 50, 56, 64].map((line) => [line, "disposition"]),
  );
});
it("proves resolvable imported consumer bodies", () => {
  assert.deepEqual(
    reports("imported.ts", true),
    [7, 9, 11, 13, 17, 19, 21, 23, 25, 28, 30, 32].map((line) => [line, "disposition"]),
  );
});
it("does not enumerate the parser program outside the resolved implementation", () => {
  const parser = {
    ...tsParser,
    parseForESLint(...args) {
      const parsed = tsParser.parseForESLint(...args);
      parsed.services.program = new Proxy(parsed.services.program, {
        get(target, key) {
          assert.notEqual(key, "getSourceFiles", "consumer proofs must stay scoped");
          const value = Reflect.get(target, key);
          return typeof value === "function" ? value.bind(target) : value;
        },
      });
      return parsed;
    },
  };
  assert.deepEqual(reports("imported.ts", true, undefined, parser), reports("imported.ts", true));
});
it("keeps imported bodies unproven without parser type information", () => {
  assert.deepEqual(
    reports("imported.ts"),
    [3, 5, 7, 9, 11, 13, 17, 19, 21, 23, 25, 28, 30, 32].map((line) => [line, "disposition"]),
  );
});
it("accepts only configured fresh dependency properties", () => {
  assert.deepEqual(
    reports("properties.ts", false, [propertyConsumer]),
    [7, 9, 11, 13, 15, 18, 20, 24, 28].map((line) => [line, "disposition"]),
  );
});
it("does not infer dependency contracts without configuration", () => {
  assert.deepEqual(
    reports("properties.ts"),
    [3, 5, 7, 9, 11, 13, 15, 18, 20, 24, 26, 28].map((line) => [line, "disposition"]),
  );
});
it("validates exact dependency property contracts", () => {
  for (const callbackArgumentProperties of [
    [],
    [{ argumentIndex: -1, propertyNames: ["enqueueEmail"] }],
    [{ argumentIndex: 0, propertyNames: [] }],
    [{ argumentIndex: 0, propertyNames: ["enqueueEmail", "enqueueEmail"] }],
    [{ argumentIndex: 0, propertyNames: [""] }],
  ]) {
    assert.throws(() =>
      reports("properties.ts", false, [{ ...propertyConsumer, callbackArgumentProperties }]),
    );
  }
});
it("matches real Oxlint for local proofs and configured imported or DI consumers", () => {
  for (const [name, lines] of [
    ["local.ts", [16, 19, 25, 27, 31, 38, 43, 50, 56, 64]],
    ["imported.ts", [7, 9, 11, 13, 17, 19, 21, 23, 25, 28, 30, 32]],
    ["properties.ts", [7, 9, 11, 13, 15, 18, 20, 24, 28]],
  ]) {
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        resolve(root, ".oxlintrc.json"),
        "--format",
        "json",
        resolve(root, name),
      ],
      { encoding: "utf8" },
    );
    assert.equal(result.status, 1, result.stderr);
    const findings = JSON.parse(result.stdout).diagnostics;
    assert.deepEqual(
      findings.map((item) => [item.labels[0].span.line, item.code]),
      lines.map((line) => [line, "no-mistakes(async-call-disposition)"]),
    );
  }
});
