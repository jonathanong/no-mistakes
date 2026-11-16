import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { describe, expect, it } from "vitest";
import { configuredFindings, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const rule = "no-consecutive-unbatched-reads";
const readMethods = ["get", "ttl", "exists"];
const clients = { clients: ["kv"], readMethods };
const imports = { imports: [{ source: "@app/cache", imported: "cache" }], readMethods };
const names = ["reads-nested-expressions", "reads-nested-boundaries"];
const expectedLines = [5, 10, 15, 20, 25, 30, 35, 41, 46, 50, 54, 59, 64, 70];
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture/client-contracts");

function findings(name, options) {
  const file = `client-contracts/${name}.ts`;
  return configuredFindings(
    fixture(file),
    { [`no-mistakes/${rule}`]: ["error", options] },
    resolve(root, `${name}.ts`),
  );
}

describe("nested awaited client reads", () => {
  it("snapshots configured nested reads and execution boundaries", async () => {
    const rows = [];
    for (const options of [clients, imports]) {
      for (const name of names) {
        const diagnostics = findings(name, options);
        assert.deepEqual(
          diagnostics.map(({ line }) => line),
          name === names[0] ? expectedLines : [],
          name,
        );
        rows.push({ file: `client-contracts/${name}.ts`, options, findings: diagnostics });
      }
    }
    await expect(snapshotDocument(rows)).toMatchFileSnapshot(
      "snapshots/nested-read-expressions.snap",
    );
  });

  it("keeps Oxlint and ESLint findings identical for the saved fixtures", () => {
    const result = spawnSync(
      process.execPath,
      [
        resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
        "--config",
        resolve(root, ".oxlintrc.nested-reads.json"),
        "--format",
        "json",
        ...names.map((name) => resolve(root, `${name}.ts`)),
      ],
      { encoding: "utf8" },
    );
    assert.equal(result.error, undefined);
    assert.equal(result.status, 1, result.stdout + result.stderr);
    const actual = JSON.parse(result.stdout).diagnostics.filter(({ code }) => code.includes(rule));
    assert.deepEqual(
      actual
        .map(({ filename, labels }) => [
          filename.replaceAll("\\", "/").split("/").at(-1),
          labels[0].span.line,
        ])
        .sort((left, right) => left[0].localeCompare(right[0]) || left[1] - right[1]),
      expectedLines.map((line) => [`${names[0]}.ts`, line]),
    );
  });
});
