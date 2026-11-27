import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname } from "./helpers.mjs";

const ROOT = resolve(
  __dirname,
  "../../../test-cases/eslint-snippets/fixture/nextjs-metadata-exports-location",
);
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const typescript = { parser: tsParser, sourceType: "module", globals: {} };

function row(name, filename, ts = false) {
  return {
    code: readFileSync(resolve(ROOT, name), "utf8"),
    filename,
    ...(ts ? { languageOptions: typescript } : {}),
  };
}

it("snapshots metadata export forms", async () => {
  await expect(
    snapshotDocument({
      "nextjs-metadata-exports-location": snapshotSuite(
        "nextjs-metadata-exports-location",
        languageOptions,
        {
          valid: [
            row("card-page.tsx", "app/cards/page.tsx", true),
            row("card-import-equals-next-line.tsx", "app/components/CardSuppressed.tsx", true),
            row("card-dynamic-next-line.js", "app/components/dynamic-suppressed.js"),
            row("card-other-export.js", "app/components/other.js"),
            row("card-local-exports.js", "app/components/local.js"),
            row("card-dynamic-page.js", "app/cards/page.js"),
          ],
          invalid: [
            row("card-import-equals.tsx", "app/components/Card.tsx", true),
            row("card-import-equals-generate.tsx", "app/components/Generate.tsx", true),
            row("card-reexport.ts", "app/components/reexport.ts", true),
            row("card-exports-metadata.js", "app/components/card.js"),
            row("card-exports-generate.js", "app/components/generate.js"),
            row("card-exports-string.js", "app/components/string.js"),
            row("card-module-exports.js", "app/components/module.js"),
            row("card-module-exports-generate.js", "app/components/module-generate.js"),
            row("card-module-spread.js", "app/components/spread.js"),
            row("card-dynamic.js", "app/components/dynamic.js"),
            row("card-dynamic-module.js", "app/components/dynamic-module.js"),
          ],
        },
      ),
    }),
  ).toMatchFileSnapshot("./snapshots/nextjs-metadata-exports-location.snap");
});
