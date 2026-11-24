import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { __dirname, fixture, plugin } from "./helpers.mjs";
import {
  oxlintFindings,
  pluginFindings,
  snapshotCases,
  snapshotDocument,
} from "./diagnostic-snapshot.mjs";

const rule = "test-no-fixed-temp-paths";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);
const files = [
  "paths.test.ts",
  "bindings.spec.cjs",
  "imports.test.ts",
  "sinks.test.ts",
  "guards.test.ts",
  "root-aliases.test.ts",
  "unsupported-paths.test.ts",
  "unique-prefixes.test.ts",
  "mock-fs.test.ts",
  "mock-node-fs.test.ts",
  "mock-fs-promises.test.ts",
  "mock-node-fs-promises.spec.cjs",
  "suppression.test.mjs",
  "suppression-file.test.mjs",
  "utility.ts",
];

it("snapshots fixed temporary path diagnostics in ESLint and Oxlint", async () => {
  const eslint = files.map((file) => {
    const sourceType = file.endsWith(".cjs") ? "commonjs" : "module";
    return {
      name: file,
      file,
      rule,
      findings: pluginFindings(fixture(`${rule}/${file}`), rule, undefined, file, {}, sourceType),
    };
  });
  const result = spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--config",
      resolve(root, ".oxlintrc.json"),
      "--format",
      "json",
      ...files.map((file) => resolve(root, file)),
    ],
    { encoding: "utf8" },
  );
  expect(result.stderr).toBe("");
  expect(result.status).toBe(1);
  const oxlint = oxlintFindings(JSON.parse(result.stdout).diagnostics);
  expect(oxlint.map(({ file, line }) => [file.split("/").at(-1), line]).sort()).toEqual(
    eslint.flatMap(({ file, findings }) => findings.map(({ line }) => [file, line])).sort(),
  );
  await expect(snapshotDocument({ eslint: snapshotCases(eslint), oxlint })).toMatchFileSnapshot(
    "snapshots/test-no-fixed-temp-paths.snap",
  );
});

it("exports the rule for opt-in use", () => {
  expect(plugin.rules[rule]).toBeDefined();
  expect(plugin.configs.recommended.rules[`no-mistakes/${rule}`]).toBeUndefined();
  expect(plugin.configs.strict.rules[`no-mistakes/${rule}`]).toBeUndefined();
});
