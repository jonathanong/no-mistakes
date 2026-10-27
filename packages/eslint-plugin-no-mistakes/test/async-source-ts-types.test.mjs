import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { it } from "vitest";
import { __dirname } from "./helpers.mjs";

const require = createRequire(import.meta.url);
const ts = require("typescript");
const { parseTypeScript } = require("../src/rules/async-source-ts-parser");

it("keeps mapped and inferred names out of runtime binding lookup", () => {
  const filename = resolve(
    __dirname,
    "../../../test-cases/eslint-plugin/async-call-disposition/fixture/optional-peers/adapter-types.ts",
  );
  const text = readFileSync(filename, "utf8");
  const runtimeGlobals = (parsed) =>
    parsed.scopeManager.scopes
      .flatMap((scope) => scope.references)
      .filter((ref) => ref.identifier.name === "Map" && ref.isValueReference)
      .map((ref) => ({
        line: ref.identifier.loc.start.line,
        locallyBound: Boolean(ref.resolved?.defs.length),
      }));
  const expected = runtimeGlobals(
    tsParser.parseForESLint(text, { filePath: filename, loc: true, range: true }),
  );
  assert.deepEqual(expected, [
    { line: 1, locallyBound: false },
    { line: 12, locallyBound: false },
  ]);
  assert.deepEqual(runtimeGlobals(parseTypeScript(ts, text, filename)), expected);
});
