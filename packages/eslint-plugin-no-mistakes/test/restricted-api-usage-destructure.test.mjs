import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, __dirname } from "./helpers.mjs";

const id = "restricted-api-usage";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
const languageOptions = { ecmaVersion: 2024, sourceType: "module", globals: {} };
const names = "^(enqueueReviewAndWait|addAndWait|waitUntilFinished)$";

function snapshotCase(name, target) {
  return {
    code: fixture(`${id}/${name}`),
    filename: resolve(root, name),
    options: [{ targets: [target] }],
    languageOptions: { parser: tsParser },
  };
}

describe(`${id} destructure and source-less imports`, () => {
  it("snapshots destructure keys and imports without a source", async () => {
    const same = { destructureNamePattern: names, message: "Use the static helper." };
    await expect(
      snapshotDocument({
        [id]: snapshotSuite(id, languageOptions, {
          valid: [
            snapshotCase("destructure.ts", { referenceNamePattern: names }),
            snapshotCase("import-sourceless.ts", {
              source: "@example/client",
              importedNamePattern: names,
            }),
          ],
          invalid: [
            snapshotCase("destructure.ts", same),
            snapshotCase("destructure.ts", { destructureNamePattern: "^add$" }),
            snapshotCase("destructure.ts", { propertyNamePattern: names }),
            snapshotCase("import-sourceless.ts", { importedNamePattern: names }),
            snapshotCase("import-sourceless.ts", {
              importedNamePattern: "^default$",
              message: "No default imports.",
            }),
            snapshotCase("import-sourceless.ts", {
              source: "@example/queue",
              importedNamePattern: names,
            }),
          ],
        }),
      }),
    ).toMatchFileSnapshot("snapshots/restricted-api-usage-destructure.snap");
  });
});
