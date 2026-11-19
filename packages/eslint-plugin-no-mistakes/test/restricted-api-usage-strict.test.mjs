import assert from "node:assert/strict";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, __dirname, lint } from "./helpers.mjs";

const id = "restricted-api-usage";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
const languageOptions = { ecmaVersion: 2024, sourceType: "module", globals: {} };
const selectors = {
  "strict-reference.ts": { referenceNamePattern: "^blockingWait$" },
  "strict-declaration.ts": { declarationNamePattern: "AndWait$" },
  "strict-property.ts": { propertyNamePattern: "^blockingWait" },
};
const construct = { constructNamePattern: "^RateLimitedWorker$", constructArgumentCount: 0 };

function snapshotCase(name, target) {
  return {
    code: fixture(`${id}/${name}`),
    filename: resolve(root, name),
    options: [{ targets: [target] }],
    languageOptions: { parser: tsParser },
  };
}

describe(`${id} strict selectors`, () => {
  it("snapshots opt-in lexical selector diagnostics", async () => {
    await expect(
      snapshotDocument({
        [id]: snapshotSuite(id, languageOptions, {
          valid: [
            snapshotCase("strict-none.ts", {
              source: "@example/client",
              importedNamePattern: "^paginate[A-Z]",
              callNamePattern: "^paginate[A-Z]",
            }),
          ],
          invalid: [
            ...Object.entries(selectors).map(([name, target]) => snapshotCase(name, target)),
            snapshotCase("strict-construct.ts", construct),
            snapshotCase("strict-construct.ts", { constructNamePattern: "Worker$" }),
            snapshotCase("strict-none.ts", {
              ...selectors["strict-reference.ts"],
              ...selectors["strict-declaration.ts"],
              ...selectors["strict-property.ts"],
              ...construct,
              message: "Name is forbidden.",
            }),
          ],
        }),
      }),
    ).toMatchFileSnapshot("snapshots/restricted-api-usage-strict.snap");
  });
  it("validates strict selector options", () => {
    const check = (target) =>
      lint(
        fixture(`${id}/strict-none.ts`),
        { [`no-mistakes/${id}`]: ["error", { targets: [target] }] },
        resolve(root, "strict-none.ts"),
      );
    assert.throws(() => check({ referenceNamePattern: "[" }), /Invalid regular expression/);
    for (const target of [
      { constructArgumentCount: 0 },
      { ...construct, constructArgumentCount: -1 },
      { message: "only" },
    ]) {
      assert.throws(() => check(target), /configuration|match|propert|should be/i);
    }
  });
});
