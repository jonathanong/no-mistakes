import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture } from "./helpers.mjs";

const snippet = (name) => fixture(name);
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const typescript = { parser: tsParser, sourceType: "module", globals: {} };

it("snapshots remaining diagnostics", async () => {
  await expect(
    snapshotDocument({
      "react-no-iife-in-jsx": snapshotSuite("react-no-iife-in-jsx", languageOptions, {
        invalid: [
          // 5 iife diagnostics. messages() default filename is fixture.jsx.
          { code: snippet("react-no-iife-in-jsx/recall.tsx"), filename: "fixture.jsx" },
        ],
      }),
      "react-no-nullish-react-node": snapshotSuite("react-no-nullish-react-node", languageOptions, {
        invalid: [
          // 4 diagnostics through static keys and const aliases.
          {
            code: snippet("react-no-nullish-react-node/recall.tsx"),
            filename: "recall.tsx",
            languageOptions: typescript,
          },
        ],
      }),
      "react-no-use-promise-resolve": snapshotSuite(
        "react-no-use-promise-resolve",
        languageOptions,
        {
          invalid: [
            // 12 diagnostics through the same-file spellings.
            {
              code: snippet("react-no-use-promise-resolve/recall.tsx"),
              filename: "recall.tsx",
              languageOptions: typescript,
            },
          ],
        },
      ),
    }),
  ).toMatchFileSnapshot("snapshots/recall-react-remaining.snap");
}, 30_000);
