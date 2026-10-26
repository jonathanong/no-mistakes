import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { it } from "vitest";
import { __dirname, lint } from "./helpers.mjs";
const code = readFileSync(
  resolve(
    __dirname,
    "../../../test-cases/eslint-plugin/async-call-disposition/fixture/awaiting-consumers.ts",
  ),
  "utf8",
);
const targets = [{ sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["enqueueEmail"] }];
const consumer = {
  sourceSpecifierPatterns: ["@app/consumer"],
  calleeNamePatterns: ["awaitReportedEnqueueAdmission"],
  callbackArgumentIndexes: [0],
};
function reports(options) {
  return lint(code, { "no-mistakes/async-call-disposition": ["error", options] }, "fixture.ts").map(
    ({ messageId, line, column }) => ({ messageId, line, column }),
  );
}
it("accepts only the configured consumer identity and callback positions", () => {
  assert.deepEqual(
    reports({ targets, awaitingConsumers: [consumer] }),
    [13, 14, 15, 16, 17, 20, 26].map((line) => ({
      messageId: "disposition",
      line,
      column: { 13: 26, 14: 15, 15: 23, 16: 19, 17: 44, 20: 20, 26: 40 }[line],
    })),
  );
});
it("keeps default behavior and does nothing without targets", () => {
  assert.deepEqual(
    reports({ targets }),
    [
      [4, 32],
      [5, 17],
      [6, 29],
      [7, 17],
      [8, 30],
      [9, 28],
      [12, 15],
      [13, 26],
      [14, 15],
      [15, 23],
      [16, 19],
      [17, 44],
      [20, 20],
      [21, 39],
      [22, 33],
      [26, 40],
    ].map(([line, column]) => ({ messageId: "disposition", line, column })),
  );
  assert.deepEqual(reports({ awaitingConsumers: [consumer] }), []);
});
it("requires explicit valid callback argument indexes", () => {
  for (const callbackArgumentIndexes of [undefined, [], [-1], [0.5], [0, 0]]) {
    const invalid = { ...consumer, callbackArgumentIndexes };
    if (callbackArgumentIndexes === undefined) delete invalid.callbackArgumentIndexes;
    assert.throws(() => reports({ targets, awaitingConsumers: [invalid] }));
  }
});
it("rejects consumer object writes including aliases without changing target matching", () => {
  const code = readFileSync(
    resolve(
      __dirname,
      "../../../test-cases/eslint-plugin/async-call-disposition/fixture/awaiting-consumer-writes.ts",
    ),
    "utf8",
  );
  for (const awaitingConsumers of [undefined, [consumer]]) {
    const result = lint(
      code,
      { "no-mistakes/async-call-disposition": ["error", { targets, awaitingConsumers }] },
      "fixture.ts",
    );
    assert.deepEqual(
      result.map(({ messageId, line, column }) => ({ messageId, line, column })),
      [
        [9, 41],
        [10, 40],
        [11, 46],
        [18, 50],
        [19, 49],
        [20, 48],
        [27, 48],
        [28, 51],
        [29, 47],
      ].map(([line, column]) => ({ messageId: "disposition", line, column })),
    );
  }
});

it("keeps source matching explicit and supports another configured slot", () => {
  assert.deepEqual(
    reports({
      targets,
      awaitingConsumers: [{ ...consumer, sourceSpecifierPatterns: ["@other/consumer"] }],
    }),
    reports({ targets }),
  );
  assert.deepEqual(
    reports({ targets, awaitingConsumers: [{ ...consumer, callbackArgumentIndexes: [1] }] }),
    reports({ targets }).filter(({ line }) => line !== 15),
  );
  assert.throws(() =>
    lint(
      code,
      { "no-mistakes/async-try-catch-return-await": ["error", { awaitingConsumers: [consumer] }] },
      "fixture.ts",
    ),
  );
});
