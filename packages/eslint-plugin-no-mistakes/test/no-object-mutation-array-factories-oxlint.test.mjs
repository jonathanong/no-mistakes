import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { __dirname } from "./helpers.mjs";
import { oxlintFindings, snapshotDocument } from "./diagnostic-snapshot.mjs";

it("preserves array factory proof boundaries through real Oxlint", async () => {
  const root = resolve(__dirname, "../../..");
  const fixtures = "test-cases/eslint-snippets/fixture";
  const result = spawnSync(
    process.execPath,
    [
      resolve(root, "node_modules/oxlint/bin/oxlint"),
      "--config",
      `${fixtures}/no-object-mutation-receiver.oxlint.json`,
      "--format",
      "json",
      ...[
        "positive.ts",
        "accept-only-from.ts",
        "accept-only-pending.ts",
        "accept-only-factory-escape.ts",
        "accept-only-factory-arguments.ts",
        "accept-only-pending-direct.ts",
        "type-names.ts",
        "typed.ts",
        "negative.ts",
        "overrides.ts",
        "global-alias.ts",
        "global-escape.ts",
        "prototype-alias.ts",
      ].map((name) => `${fixtures}/no-object-mutation-array-factories/${name}`),
    ],
    { cwd: root, encoding: "utf8" },
  );
  expect(result.error, result.stderr).toBeUndefined();
  expect(result.status, result.stderr).toBe(1);
  const { diagnostics } = JSON.parse(result.stdout);
  expect(diagnostics.length).toBeGreaterThan(0);
  expect(
    diagnostics.filter((finding) =>
      /(?:positive|type-names|accept-only-from|accept-only-factory-escape)\.ts$/.test(
        finding.filename,
      ),
    ),
  ).toEqual([]);
  await expect(snapshotDocument(oxlintFindings(diagnostics))).toMatchFileSnapshot(
    "snapshots/no-object-mutation-array-factories-oxlint.snap",
  );
});
