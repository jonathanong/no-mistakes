import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { oxlintFindings, snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture, messages } from "./helpers.mjs";

const rule = "no-object-mutation";
const arrays = "no-object-mutation-array-receivers.ts";
const imported = "no-object-mutation-array-receiver-import.ts";
const controls = "no-object-mutation-array-receiver-controls.ts";
const languageOptions = { ecmaVersion: 2024, sourceType: "module", parser: tsParser };
const savedCase = (name, options) => ({ code: fixture(name), filename: name, options });

describe("no-object-mutation array receiver provenance", () => {
  it("reports inherited arrays, tuples, and global Array types when allowance is disabled", () => {
    expect(messages(fixture(arrays), rule, { allowThis: false }, arrays)).toEqual(
      Array.from({ length: 8 }, () => "mutation"),
    );
  });

  it("allows array receivers by default and keeps custom Array types callable", () => {
    expect(messages(fixture(arrays), rule, undefined, arrays)).toEqual([]);
    expect(messages(fixture(controls), rule, { allowThis: false }, controls)).toEqual([]);
  });

  it("snapshots intrinsic arrays and shadowed type controls", async () => {
    await expect(
      snapshotDocument(
        snapshotSuite(rule, languageOptions, {
          valid: [
            savedCase(arrays),
            savedCase(controls, [{ allowThis: false }]),
            savedCase(imported, [{ allowThis: false }]),
          ],
          invalid: [savedCase(arrays, [{ allowThis: false }])],
        }),
      ),
    ).toMatchFileSnapshot("snapshots/no-object-mutation-array-receivers.snap");
  });
  it("preserves receiver type scopes through Oxlint", async () => {
    const repoRoot = resolve(__dirname, "../../..");
    const fixtureRoot = "test-cases/eslint-snippets/fixture";
    const result = spawnSync(
      process.execPath,
      [
        resolve(repoRoot, "node_modules/oxlint/bin/oxlint"),
        "--config",
        `${fixtureRoot}/no-object-mutation-array-receivers.oxlint.json`,
        "--format",
        "json",
        ...[arrays, controls, imported].map((name) => `${fixtureRoot}/${name}`),
      ],
      { cwd: repoRoot, encoding: "utf8" },
    );
    expect(result.error, result.stderr).toBeUndefined();
    expect(result.status, result.stderr).toBe(1);
    const { diagnostics } = JSON.parse(result.stdout);
    expect(diagnostics).toHaveLength(8);
    await expect(snapshotDocument(oxlintFindings(diagnostics))).toMatchFileSnapshot(
      "snapshots/no-object-mutation-array-receivers-oxlint.snap",
    );
  });
});
