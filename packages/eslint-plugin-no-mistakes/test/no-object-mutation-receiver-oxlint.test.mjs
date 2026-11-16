import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { __dirname } from "./helpers.mjs";
import { oxlintFindings, snapshotDocument } from "./diagnostic-snapshot.mjs";

const repoRoot = resolve(__dirname, "../../..");
const fixtureRoot = "test-cases/eslint-snippets/fixture";

describe("no-object-mutation receiver Oxlint compatibility", () => {
  it("preserves declared receivers and computed builders through the real plugin", async () => {
    const result = spawnSync(
      process.execPath,
      [
        resolve(repoRoot, "node_modules/oxlint/bin/oxlint"),
        "--config",
        `${fixtureRoot}/no-object-mutation-receiver.oxlint.json`,
        "--format",
        "json",
        ...[
          "no-object-mutation-receiver.ts",
          "no-object-mutation-receiver.js",
          "no-object-mutation-computed-local.ts",
        ].map((name) => `${fixtureRoot}/${name}`),
      ],
      { cwd: repoRoot, encoding: "utf8" },
    );
    expect(result.error, result.stderr).toBeUndefined();
    expect(result.status, result.stderr).toBe(1);
    const { diagnostics } = JSON.parse(result.stdout);
    expect(diagnostics).toHaveLength(1);
    expect(diagnostics[0].code).toBe("no-mistakes(no-object-mutation)");
    await expect(snapshotDocument(oxlintFindings(diagnostics))).toMatchFileSnapshot(
      "snapshots/no-object-mutation-receiver-oxlint.snap",
    );
  });
});
