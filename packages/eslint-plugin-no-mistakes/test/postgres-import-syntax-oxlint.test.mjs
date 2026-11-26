import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { oxlintFindings, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { __dirname } from "./helpers.mjs";

const repoRoot = resolve(__dirname, "../../..");
const fixtureRoot = "test-cases/eslint-snippets/fixture/postgres-import-syntax";

it("snapshots PostgreSQL import and wrapper diagnostics through Oxlint", async () => {
  const result = spawnSync(
    process.execPath,
    [
      resolve(repoRoot, "node_modules/oxlint/bin/oxlint"),
      "--config",
      `${fixtureRoot}/.oxlintrc.json`,
      "--format",
      "json",
      fixtureRoot,
    ],
    { cwd: repoRoot, encoding: "utf8", timeout: 20_000 },
  );

  expect(result.error, result.stderr).toBeUndefined();
  expect(result.status, result.stderr).toBe(1);
  const findings = oxlintFindings(JSON.parse(result.stdout).diagnostics);
  expect(new Set(findings.map(({ code }) => code))).toEqual(
    new Set([
      "no-mistakes(postgres-cursor-call-contract)",
      "no-mistakes(postgres-no-async-sql-builder)",
      "no-mistakes(postgres-no-executor-adapter)",
      "no-mistakes(postgres-no-manual-transaction)",
      "no-mistakes(postgres-no-nonliteral-sql-registry-key)",
      "no-mistakes(postgres-no-unbounded-query-fanout)",
      "no-mistakes(postgres-pool-requires-error-listener)",
    ]),
  );
  await expect(snapshotDocument(findings)).toMatchFileSnapshot(
    "snapshots/postgres-import-syntax-oxlint.snap",
  );
});
