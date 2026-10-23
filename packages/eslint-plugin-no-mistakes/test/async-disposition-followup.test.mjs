import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { describe, it } from "vitest";
import { __dirname, messages } from "./helpers.mjs";
const options = {
  targets: [{ sourceSpecifierPatterns: ["@app/jobs"], calleeNamePatterns: ["/^enqueue[A-Z].*/"] }],
};
function reports(name) {
  const code = readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin/async-call-disposition/fixture", name),
    "utf8",
  );
  return messages(code, "async-call-disposition", options, name);
}
describe("async disposition follow-up saved recall", () => {
  for (const name of [
    "followup-collections.ts",
    "followup-mocked.ts",
    "downstream-adoption.ts",
    "downstream-conditional-return.ts",
    "downstream-let-spy.ts",
  ]) {
    it(`accepts ${name}`, () => assert.deepEqual(reports(name), []));
  }
  it("keeps independent unhandled controls reported", () =>
    assert.deepEqual(reports("followup-controls.ts"), Array(21).fill("disposition")));
  it("keeps array identity, early exits, rewrites, and unknown adoption reported", () => {
    assert.deepEqual(reports("followup-contract-controls.ts"), Array(20).fill("disposition"));
    assert.deepEqual(reports("followup-rewritten-promise.ts"), ["disposition"]);
  });
});
