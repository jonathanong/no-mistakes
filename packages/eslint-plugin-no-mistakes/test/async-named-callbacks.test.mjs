import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { it } from "vitest";
import { __dirname, lint } from "./helpers.mjs";

const code = readFileSync(
  resolve(
    __dirname,
    "../../../test-cases/eslint-plugin/async-call-disposition/fixture/named-callbacks.ts",
  ),
  "utf8",
);
const targets = [{ sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["enqueueEmail"] }];
const awaitingConsumers = [
  {
    sourceSpecifierPatterns: ["@app/consumer"],
    calleeNamePatterns: ["awaitReportedEnqueueAdmission"],
    callbackArgumentIndexes: [0],
  },
];

function reports(options) {
  return lint(code, { "no-mistakes/async-call-disposition": ["error", options] }, "named.ts").map(
    ({ line, messageId }) => [line, messageId],
  );
}

it("checks stable named callbacks at every argument use", () => {
  assert.deepEqual(
    reports({ targets, awaitingConsumers }),
    [5, 7, 9, 16, 18, 20, 28, 30, 37, 48, 50, 57, 59, 61, 68, 77, 80, 92, 94, 96].map((line) => [
      line,
      "disposition",
    ]),
  );
});

it("keeps named callback returns unproven without an awaiting consumer", () => {
  assert.deepEqual(
    reports({ targets }),
    [
      5, 7, 9, 12, 14, 16, 18, 20, 23, 28, 30, 37, 39, 41, 43, 48, 50, 53, 55, 57, 59, 61, 68, 77,
      80, 85, 92, 94, 96,
    ].map((line) => [line, "disposition"]),
  );
});

it("distinguishes callback values from expressions built from callbacks", () => {
  const valueUses = readFileSync(
    resolve(
      __dirname,
      "../../../test-cases/eslint-plugin/async-call-disposition/fixture/named-callback-value-uses.ts",
    ),
    "utf8",
  );
  const result = lint(
    valueUses,
    { "no-mistakes/async-call-disposition": ["error", { targets, awaitingConsumers }] },
    "value-uses.ts",
  );
  assert.deepEqual(
    result.map(({ line, messageId }) => [line, messageId]),
    [24, 26, 28, 30, 32, 35].map((line) => [line, "disposition"]),
  );
});

it("preserves a CommonJS top-level return as an ordinary return", () => {
  const commonjs = readFileSync(
    resolve(
      __dirname,
      "../../../test-cases/eslint-plugin/async-call-disposition/fixture/commonjs-top-level-return.cjs",
    ),
    "utf8",
  );
  const result = lint(
    commonjs,
    { "no-mistakes/async-call-disposition": ["error", { targets }] },
    "return.cjs",
    {},
    "commonjs",
  );
  assert.deepEqual(
    result.map(({ messageId }) => messageId),
    [],
  );
});

it("preserves anonymous default function returns", () => {
  const anonymous = readFileSync(
    resolve(
      __dirname,
      "../../../test-cases/eslint-plugin/async-call-disposition/fixture/anonymous-default-return.mjs",
    ),
    "utf8",
  );
  const result = lint(
    anonymous,
    { "no-mistakes/async-call-disposition": ["error", { targets }] },
    "default.mjs",
  );
  assert.deepEqual(
    result.map(({ messageId }) => messageId),
    [],
  );
});
