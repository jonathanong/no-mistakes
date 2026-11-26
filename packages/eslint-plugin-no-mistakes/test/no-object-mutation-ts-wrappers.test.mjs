import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import tsParser from "@typescript-eslint/parser";
import { oxlintFindings, snapshotDocument, snapshotSuite } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const repoRoot = resolve(__dirname, "../../..");
const fixtureRoot = "test-cases/eslint-snippets/fixture";
const filename = "no-object-mutation-ts-wrappers.ts";

describe("no-object-mutation TypeScript assignment wrappers", () => {
  it("reports writes through TS wrappers in ESLint", async () => {
    await expect(
      snapshotDocument(
        snapshotSuite(
          "no-object-mutation",
          { ecmaVersion: 2024, sourceType: "module", parser: tsParser },
          { invalid: [{ code: fixture(filename), filename }] },
        ),
      ),
    ).toMatchFileSnapshot("snapshots/no-object-mutation-ts-wrappers.snap");
  });

  it("reports the same wrapped writes through real Oxlint", async () => {
    const result = spawnSync(
      process.execPath,
      [
        resolve(repoRoot, "node_modules/oxlint/bin/oxlint"),
        "--config",
        `${fixtureRoot}/no-object-mutation-receiver.oxlint.json`,
        "--format",
        "json",
        `${fixtureRoot}/${filename}`,
      ],
      { cwd: repoRoot, encoding: "utf8" },
    );
    expect(result.error, result.stderr).toBeUndefined();
    expect(result.status, result.stderr).toBe(1);
    const { diagnostics } = JSON.parse(result.stdout);
    expect(diagnostics.length).toBeGreaterThan(0);
    expect(diagnostics.every((finding) => finding.code === "no-mistakes(no-object-mutation)")).toBe(
      true,
    );
    await expect(snapshotDocument(oxlintFindings(diagnostics))).toMatchFileSnapshot(
      "snapshots/no-object-mutation-ts-wrappers-oxlint.snap",
    );
  });
});
