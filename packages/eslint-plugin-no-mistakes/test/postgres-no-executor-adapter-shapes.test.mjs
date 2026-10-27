import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { Linter } from "eslint";
import { describe, expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { fixture, __dirname, require } from "./helpers.mjs";

const RULE = "postgres-no-executor-adapter";
const id = RULE;
const DB = { importSpecifier: "@example/db" };
const SCOPED = {
  importSpecifier: "@example/db",
  executorFactoryNames: ["openTx"],
  executorTypeNames: ["Tx"],
};
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", id);
const rules = resolve(__dirname, "../src/rules");
const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};

function shapeCase(name, option = DB) {
  return {
    code: fixture(`${id}/${name}`),
    filename: resolve(root, name),
    options: [option],
    ...(/\.[cm]?tsx?$/.test(name) ? { languageOptions: { parser: tsParser } } : {}),
  };
}

const suite = {
  valid: [
    // leaves non-static executor aliases silent.
    // Zero findings.
    shapeCase("static-recall-limits.ts", SCOPED),
    // leaves unjudged shapes silent, including an untyped client.query.
    // Zero findings.
    shapeCase("silent.ts"),
    // Zero findings.
    shapeCase("names.ts", { importSpecifier: "@example/db", executorNames: ["read"] }),
  ],
  invalid: [
    // reports the shapes whose rewrite is the executor call.
    // Eight findings.
    shapeCase("shapes.ts"),
    // names only the executor that the expression can pass.
    shapeCase("edges.ts"),
    // reports namespace members, query destructures, and static call or apply.
    shapeCase("static-recall.ts", SCOPED),
    // Two findings.
    shapeCase("call-method.ts", { importSpecifier: "@example/db", executorNames: ["call"] }),
    // follows a proven query member through bind and new, and a broad query name.
    // Two passed findings.
    shapeCase("scoped.ts", SCOPED),
    // Four findings. broad.mjs is not parsed as TypeScript.
    shapeCase("broad.mjs", { executorNames: ["query"] }),
    // keeps the existing suppression.
    // One finding.
    shapeCase("suppression.ts"),
  ],
};

it("snapshots postgres-no-executor-adapter diagnostics", async () => {
  await expect(snapshotDocument(snapshotSuite(RULE, languageOptions, suite))).toMatchFileSnapshot(
    "snapshots/postgres-no-executor-adapter-shapes.snap",
  );
}, 30_000);

describe("postgres-no-executor-adapter shapes", () => {
  it("does not autofix", () => {
    const linter = new Linter({ configType: "flat", cwd: root });
    const code = fixture(`${id}/shapes.ts`);
    const fixed = linter.verifyAndFix(
      code,
      [
        {
          files: ["**/*.{js,jsx,ts,tsx,mjs,mts,cjs,cts}"],
          languageOptions: { ecmaVersion: 2024, sourceType: "module", parser: tsParser },
          plugins: { "no-mistakes": require("../src/index.js") },
          rules: { [`no-mistakes/${RULE}`]: ["error", DB] },
        },
      ],
      resolve(root, "shapes.ts"),
    );
    assert.equal(fixed.output, code);
  });

  it("imports binding helpers without a ref and member cycle", () => {
    const ref = readFileSync(resolve(rules, "postgres-no-executor-adapter-ref.js"), "utf8");
    const member = readFileSync(resolve(rules, "postgres-no-executor-adapter-member.js"), "utf8");
    const binding = readFileSync(resolve(rules, "postgres-no-executor-adapter-binding.js"), "utf8");
    const body = ref.slice(ref.indexOf("function executorRef"));
    assert.equal(body.includes("require("), false);
    assert.match(ref, /require\("\.\/postgres-no-executor-adapter-binding"\)/);
    assert.match(ref, /require\("\.\/postgres-no-executor-adapter-member"\)/);
    assert.doesNotMatch(member, /postgres-no-executor-adapter-ref/);
    assert.doesNotMatch(binding, /postgres-no-executor-adapter-ref/);
    assert.doesNotMatch(binding, /postgres-no-executor-adapter-member/);
  });
});
