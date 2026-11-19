import assert from "node:assert/strict";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, __dirname, lint } from "./helpers.mjs";

const id = "restricted-api-usage";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
const base = { callNamePattern: "^goto$", message: "Capture or return the result." };
const opt = (target) => ({ targets: [target] });
const languageOptions = { ecmaVersion: 2024, sourceType: "module", globals: {} };

function check(options) {
  const name = "discarded.ts";
  const rules = { [`no-mistakes/${id}`]: ["error", options] };
  return lint(fixture(`${id}/${name}`), rules, resolve(root, name));
}
function checkExtra(options) {
  const name = "discarded-extra.ts";
  const rules = { [`no-mistakes/${id}`]: ["error", options] };
  return lint(fixture(`${id}/${name}`), rules, resolve(root, name));
}
function snapshotCase(options) {
  return {
    code: fixture(`${id}/discarded.ts`),
    filename: resolve(root, "discarded.ts"),
    options: [options],
    languageOptions: { parser: tsParser },
  };
}

describe(`${id} discardedResultOnly`, () => {
  it("snapshots result-use diagnostics with the option on and off", async () => {
    await expect(
      snapshotDocument({
        [id]: snapshotSuite(id, languageOptions, {
          valid: [],
          invalid: [
            snapshotCase(opt({ ...base, discardedResultOnly: true })),
            snapshotCase(opt(base)),
          ],
        }),
      }),
    ).toMatchFileSnapshot("snapshots/restricted-api-usage-discarded.snap");
  });
  it("reports only discarded results and keeps the default unchanged", () => {
    const on = check(opt({ ...base, discardedResultOnly: true }));
    assert.deepEqual(
      on.map((message) => message.line),
      [6, 7, 8, 9, 9, 10, 11, 12, 23],
    );
    assert.equal(check(opt(base)).length, 16);
    assert.equal(check(opt({ ...base, discardedResultOnly: false })).length, 16);
  });
  it("treats generic instantiations as transparent and keeps import origins independent", () => {
    const lines = (options) => checkExtra(opt(options)).map((message) => message.line);
    assert.deepEqual(lines({ callNamePattern: "^factory$", discardedResultOnly: true }), [5]);
    assert.deepEqual(
      lines({
        source: "client",
        importedNamePattern: "^fetch$",
        callNamePattern: "^goto$",
        discardedResultOnly: true,
      }),
      [1, 7],
    );
  });
  it("requires a lexical call selector", () => {
    for (const bad of [
      { discardedResultOnly: true },
      { source: "@example/client", importedNamePattern: "^x", discardedResultOnly: true },
    ]) {
      assert.throws(() => check(opt(bad)), /configuration|match|propert/i);
    }
  });
});
