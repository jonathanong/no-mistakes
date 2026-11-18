import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { __dirname } from "./helpers.mjs";
import { oxlintFindings, snapshotDocument } from "./diagnostic-snapshot.mjs";

it("preserves synchronous array callback ownership through real Oxlint", async () => {
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
        "synchronous-controls.js",
        "typed-controls.ts",
        "synchronous-escapes.js",
        "typed-escapes.ts",
      ].map((name) => `${fixtures}/no-object-mutation-iteration-escapes/${name}`),
    ],
    { cwd: root, encoding: "utf8" },
  );
  expect(result.error, result.stderr).toBeUndefined();
  expect(result.status, result.stderr).toBe(1);
  const { diagnostics } = JSON.parse(result.stdout);
  expect(diagnostics.length).toBeGreaterThan(0);
  expect(diagnostics.filter((finding) => finding.filename.includes("controls."))).toEqual([]);
  await expect(snapshotDocument(oxlintFindings(diagnostics))).toMatchFileSnapshot(
    "snapshots/no-object-mutation-synchronous-oxlint.snap",
  );
});
