import assert from "node:assert/strict";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, __dirname, lint } from "./helpers.mjs";

const id = "restricted-api-usage";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
const target = { callNamePattern: "^goto$", message: "Capture or return the result." };
const languageOptions = { ecmaVersion: 2024, sourceType: "module", globals: {} };

function check(name, discardedResultOnly) {
  const options = { targets: [{ ...target, discardedResultOnly }] };
  const rules = { [`no-mistakes/${id}`]: ["error", options] };
  return lint(fixture(`${id}/${name}`), rules, resolve(root, name));
}
function snapshotCase(name, discardedResultOnly) {
  return {
    code: fixture(`${id}/${name}`),
    filename: resolve(root, name),
    options: [{ targets: [{ ...target, discardedResultOnly }] }],
    languageOptions: { parser: tsParser },
  };
}

describe(`${id} discardedResultOnly value-discarding positions`, () => {
  it("snapshots logical, conditional, for-clause and promise-chain fixtures", async () => {
    const cases = {
      valid: [
        snapshotCase("discarded-promise-chain.ts", true),
        snapshotCase("discarded-left-operand.ts", true),
      ],
      invalid: [snapshotCase("discarded-positions.ts", true)],
    };
    await expect(
      snapshotDocument({ [id]: snapshotSuite(id, languageOptions, cases) }),
    ).toMatchFileSnapshot("snapshots/restricted-api-usage-discarded-positions.snap");
  });
  it("reports calls in logical, conditional, sequence and for-clause discards only", () => {
    const lines = check("discarded-positions.ts", true).map((message) => message.line);
    assert.deepEqual(lines, [8, 9, 10, 11, 11, 12, 13, 14, 15, 16, 16, 17, 17, 18]);
  });
  it("counts left operands of &&, || and ?? as used tests", () => {
    assert.deepEqual(check("discarded-left-operand.ts", true), []);
    assert.equal(check("discarded-left-operand.ts", false).length, 7);
  });
  it("counts promise-chain receivers as used", () => {
    assert.deepEqual(check("discarded-promise-chain.ts", true), []);
    assert.equal(check("discarded-promise-chain.ts", false).length, 4);
  });
});
