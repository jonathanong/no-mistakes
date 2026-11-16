import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const rule = "no-object-mutation";
const languageOptions = { ecmaVersion: 2024, sourceType: "module", parser: tsParser };

function savedCase(name, languageOptions, extension = "js") {
  const filename = `no-object-mutation-early-returns/${name}.${extension}`;
  return { code: fixture(filename), filename, languageOptions };
}

describe("no-object-mutation early returns", () => {
  it("keeps fresh builders local and reports writes after escape", async () => {
    await expect(
      snapshotDocument(
        snapshotSuite(rule, languageOptions, {
          valid: [savedCase("valid"), savedCase("typed", undefined, "ts")],
          invalid: [
            savedCase("escapes"),
            savedCase("top-level-return", {
              sourceType: "script",
              parserOptions: { ecmaFeatures: { globalReturn: true } },
            }),
          ],
        }),
      ),
    ).toMatchFileSnapshot("snapshots/no-object-mutation-early-returns.snap");
  });
});
