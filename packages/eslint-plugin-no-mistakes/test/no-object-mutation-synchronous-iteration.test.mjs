import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const languageOptions = { ecmaVersion: 2024, sourceType: "module", parser: tsParser };

function savedCase(name, extension = "js") {
  const filename = `no-object-mutation-iteration-escapes/${name}.${extension}`;
  return { code: fixture(filename), filename };
}

describe("no-object-mutation synchronous iteration", () => {
  it("reports shared and escaping writes while allowing local accumulation", async () => {
    await expect(
      snapshotDocument(
        snapshotSuite("no-object-mutation", languageOptions, {
          valid: [savedCase("synchronous-controls"), savedCase("typed-controls", "ts")],
          invalid: [savedCase("synchronous-escapes"), savedCase("typed-escapes", "ts")],
        }),
      ),
    ).toMatchFileSnapshot("snapshots/no-object-mutation-synchronous-iteration.snap");
  });
});
