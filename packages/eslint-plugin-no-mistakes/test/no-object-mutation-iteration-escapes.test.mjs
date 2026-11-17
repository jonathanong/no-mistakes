import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const languageOptions = { ecmaVersion: 2024, sourceType: "module", parser: tsParser };
function savedCase(name) {
  const filename = `no-object-mutation-iteration-escapes/${name}.js`;
  return { code: fixture(filename), filename };
}

describe("no-object-mutation iteration escapes", () => {
  it("reports shared builders across iterations and returning callbacks", async () => {
    await expect(
      snapshotDocument(
        snapshotSuite("no-object-mutation", languageOptions, {
          valid: [savedCase("controls")],
          invalid: [savedCase("loop"), savedCase("callbacks")],
        }),
      ),
    ).toMatchFileSnapshot("snapshots/no-object-mutation-iteration-escapes.snap");
  });
});
