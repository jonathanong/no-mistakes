import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname } from "./helpers.mjs";

const typescript = { parser: tsParser, sourceType: "module", globals: {} };

function saved(rule, name) {
  return readFileSync(
    resolve(__dirname, "../../../test-cases/eslint-plugin", rule, "fixture", name),
    "utf8",
  );
}

function lintCase(rule, name, option) {
  const item = { code: saved(rule, name), filename: name, languageOptions: typescript };
  if (option !== undefined) item.options = [option];
  return item;
}

describe("TypeScript alias import-equals and CommonJS exports", () => {
  it("snapshots import-equals and CommonJS alias diagnostics", async () => {
    await expect(
      snapshotDocument({
        "ts-no-const-aliases": snapshotSuite("ts-no-const-aliases", typescript, {
          valid: [lintCase("ts-no-const-aliases", "import-equals-same.ts")],
          invalid: [
            lintCase("ts-no-const-aliases", "import-equals.ts"),
            lintCase("ts-no-const-aliases", "import-equals.ts", { unanalyzable: "ignore" }),
            lintCase("ts-no-const-aliases", "import-equals-suppressed.ts"),
          ],
        }),
        "ts-no-export-renaming": snapshotSuite("ts-no-export-renaming", typescript, {
          valid: [
            lintCase("ts-no-export-renaming", "export-import-same.ts"),
            lintCase("ts-no-export-renaming", "export-star.ts"),
            lintCase("ts-no-export-renaming", "cjs-local.ts"),
            lintCase("ts-no-export-renaming", "export-import-opaque.ts", {
              unanalyzable: "ignore",
            }),
          ],
          invalid: [
            lintCase("ts-no-export-renaming", "export-import.ts"),
            lintCase("ts-no-export-renaming", "export-import-opaque.ts"),
            lintCase("ts-no-export-renaming", "export-import-suppressed.ts"),
            lintCase("ts-no-export-renaming", "cjs-aliases.ts"),
            lintCase("ts-no-export-renaming", "cjs-suppressed.ts"),
          ],
        }),
      }),
    ).toMatchFileSnapshot("snapshots/ts-alias-bindings.snap");
  });
});
