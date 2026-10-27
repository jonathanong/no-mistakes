import { dirname, resolve } from "node:path";
import tsParser from "@typescript-eslint/parser";
import { expect, it } from "vitest";
import { snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const languageOptions = {
  ecmaVersion: 2024,
  sourceType: "module",
  globals: {},
  parserOptions: { ecmaFeatures: { jsx: true } },
};
const option = { importSpecifier: "@example/db" };

function castCase(name) {
  const filename = resolve(
    __dirname,
    `../../../test-cases/eslint-snippets/fixture/executor-collection-key/${name}`,
  );
  return {
    code: fixture(`executor-collection-key/${name}`),
    filename,
    options: [option],
    languageOptions: {
      parser: tsParser,
      parserOptions: { project: resolve(dirname(filename), "tsconfig.json") },
    },
  };
}

const suite = {
  valid: [
    // invalidates typed collection methods written through assertions.
    // Zero findings.
    castCase("cast-write-shadowed-globalthis.ts"),
  ],
  invalid: [
    // invalidates typed collection methods written through assertions.
    // One passed finding.
    castCase("cast-write-direct.ts"),
    castCase("cast-write-delete.ts"),
    castCase("cast-write-computed.ts"),
    castCase("cast-write-dynamic-member.ts"),
    castCase("cast-write-proto.ts"),
    castCase("cast-write-destructure.ts"),
    castCase("cast-write-constructor-prototype.ts"),
    castCase("cast-write-iteration.ts"),
    castCase("cast-write-globalthis.ts"),
    castCase("cast-write-dynamic-globalthis.ts"),
    castCase("cast-write-source-facts.ts"),
  ],
};

it("snapshots postgres-no-executor-adapter diagnostics", async () => {
  await expect(
    snapshotDocument(snapshotSuite("postgres-no-executor-adapter", languageOptions, suite)),
  ).toMatchFileSnapshot("snapshots/postgres-no-executor-adapter-collection-cast-writes.snap");
}, 30_000);
