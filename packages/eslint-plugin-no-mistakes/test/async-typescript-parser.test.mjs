import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { resolve } from "node:path";
import { it } from "vitest";
import ts from "typescript";
import { fixture } from "./async-promise-handoff-real.mjs";
const require = createRequire(import.meta.url);
const { parseTypeScript } = require("../src/rules/async-source-ts-parser");
const root = resolve(fixture, "optional-peers/unsupported");
it.each([
  "class.ts",
  "this.ts",
  "using.ts",
  "modifier.ts",
  "jsx.tsx",
  "invalid.ts",
  "import-attributes.ts",
  "export-attributes.ts",
  "label.ts",
  "shorthand.ts",
  "assignment-method.ts",
])("rejects unsupported implementations without erasing runtime uses (%s)", (name) => {
  const filename = resolve(root, name);
  assert.throws(
    () => parseTypeScript(ts, readFileSync(filename, "utf8"), filename),
    /Unsupported|Invalid/,
  );
});
it("rejects a missing implementation", () => {
  assert.throws(() => parseTypeScript(ts, undefined, "missing.ts"), /Missing implementation/);
});
