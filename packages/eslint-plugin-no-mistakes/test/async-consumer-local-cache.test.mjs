import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { it } from "vitest";
import { __dirname, getLinter, plugin, runOxlint } from "./helpers.mjs";

const root = resolve(
  __dirname,
  "../../../test-cases/eslint-plugin/async-call-disposition/fixture/proven-consumers",
);
const fixtures = [
  "local-cached-consumer.ts",
  ...[
    "set",
    "get",
    "has",
    "global",
    "holder",
    "alias",
    "reflective",
    "reflect-set",
    "prototype",
    "reflect-alias",
    "holder-reflect",
    "holder-alias",
    "shadowed",
    "eval",
    "function",
    "holder-eval",
    "computed-holder",
  ].map((name) => `local-cached-map-${name}.ts`),
];

let cachedOxlint;
function oxlintBatch() {
  if (!cachedOxlint) {
    cachedOxlint = runOxlint({
      config: resolve(root, ".oxlintrc-proven.json"),
      files: fixtures.map((fixture) => resolve(root, fixture)),
      typeAware: true,
      denyWarnings: true,
    });
    assert.equal(cachedOxlint.status, 1, cachedOxlint.stderr);
  }
  return cachedOxlint;
}

for (const fixture of fixtures) {
  const expected = readFileSync(resolve(root, fixture), "utf8")
    .split("\n")
    .flatMap((line, index) => (line.includes("// report") ? [index + 1] : []));
  it.each([false, true])(
    `${fixture}: accepts only locally awaited cache consumers (typed=%s)`,
    (typed) => {
      const filename = resolve(root, fixture);
      const messages = getLinter(dirname(filename)).verify(
        readFileSync(filename, "utf8"),
        {
          files: ["**/*.ts"],
          languageOptions: {
            parser: tsParser,
            parserOptions: typed
              ? {
                  project: resolve(root, "tsconfig.json"),
                  disallowAutomaticSingleRunInference: true,
                }
              : {},
          },
          plugins: { "no-mistakes": plugin },
          rules: {
            "no-mistakes/async-call-disposition": [
              "error",
              {
                targets: [
                  { sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["enqueueEmail"] },
                ],
              },
            ],
          },
        },
        { filename },
      );
      assert.deepEqual(
        messages.map(({ line, messageId }) => [line, messageId]),
        expected.map((line) => [line, "disposition"]),
      );
    },
  );
  it(`${fixture}: checks cached consumer disposition under real type-aware Oxlint`, () => {
    const diagnostics = oxlintBatch().forFile(resolve(root, fixture));
    assert.deepEqual(
      diagnostics.map((item) => [item.labels[0].span.line, item.code]),
      expected.map((line) => [line, "no-mistakes(async-call-disposition)"]),
    );
  });
}
