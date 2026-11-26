import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { oxlintFindings, pluginFindings, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { __dirname, fixture } from "./helpers.mjs";

const fixtureName = "g4-mock-callees";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", fixtureName);
const boundary = "module-mock-boundary";
const cases = [
  ["boundary.ts", boundary, [5, 6, 7, 8, 9, 10]],
  ["preserve.ts", "module-mock-preserve-exports", [4, 5, 6, 7, 8, 9]],
];

it("checks direct and forwarded mock calls through TypeScript wrappers", async () => {
  const eslint = cases.flatMap(([file, rule, lines]) => {
    const rows = pluginFindings(fixture(`${fixtureName}/${file}`), rule, {}, file);
    expect(
      rows.map((row) => row.line),
      rule,
    ).toEqual(lines);
    return rows;
  });
  const result = spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--config",
      resolve(root, ".oxlintrc.json"),
      "--format",
      "json",
      ...cases.map(([file]) => resolve(root, file)),
    ],
    { cwd: root, encoding: "utf8" },
  );
  expect(result.error).toBeUndefined();
  expect(result.status, result.stderr).toBe(1);
  const oxlint = oxlintFindings(JSON.parse(result.stdout).diagnostics);
  expect(oxlint.every((row) => row.code?.includes("no-mistakes("))).toBe(true);
  for (const [file, , lines] of cases) {
    expect(oxlint.filter((row) => row.file.endsWith(file)).map((row) => row.line)).toEqual(lines);
  }
  await expect(snapshotDocument({ eslint, oxlint })).toMatchFileSnapshot(
    "snapshots/module-mock-callees.snap",
  );
});

it("respects the boundary policy for opaque specifiers", () => {
  const rows = pluginFindings(
    fixture(`${fixtureName}/boundary.ts`),
    boundary,
    { requireLiteralSpecifiers: false },
    "boundary.ts",
  );
  expect(rows).toEqual([]);
});
