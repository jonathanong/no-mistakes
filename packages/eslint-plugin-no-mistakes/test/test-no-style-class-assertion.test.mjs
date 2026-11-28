import { spawnSync } from "node:child_process";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import {
  oxlintFindings,
  pluginFindings,
  snapshotCases,
  snapshotDocument,
} from "./diagnostic-snapshot.mjs";
import { __dirname, fixture, oxlintBin, plugin } from "./helpers.mjs";

const rule = "test-no-style-class-assertion";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);

const cases = [
  { file: "forms.test.tsx", count: 16 },
  { file: "negated.test.tsx", count: 7 },
  { file: "receivers.test.tsx", count: 20 },
  { file: "imports.spec.ts", count: 6 },
  { file: "commonjs.test.cjs", count: 4 },
  { file: "counterexamples.test.tsx", count: 0 },
  { file: "playwright.spec.ts", count: 0 },
  { file: "non-test.tsx", count: 0 },
  { file: "suppressions.test.tsx", count: 3 },
  { file: "suppressed-file.test.tsx", count: 0 },
  { file: "suppressed-file-no-reason.test.tsx", count: 1 },
];

it("snapshots style class assertion diagnostics in ESLint and Oxlint", async () => {
  const eslint = cases.map(({ file, count }) => {
    const sourceType = file.endsWith(".cjs") ? "commonjs" : "module";
    const findings = pluginFindings(
      fixture(`${rule}/${file}`),
      rule,
      undefined,
      file,
      {},
      sourceType,
    );
    expect(
      findings.map(({ messageId }) => messageId),
      file,
    ).toEqual(Array(count).fill("styleClassAssertion"));
    return {
      name: file,
      file,
      rule,
      findings,
      ...(sourceType === "commonjs" ? { sourceType } : {}),
    };
  });

  const result = spawnSync(
    process.execPath,
    [
      oxlintBin,
      "--config",
      resolve(root, ".oxlintrc.json"),
      "--format",
      "json",
      "--no-ignore",
      ...cases.map(({ file }) => resolve(root, file)),
    ],
    { encoding: "utf8", timeout: 15_000 },
  );
  expect(result.error, result.stderr).toBeUndefined();
  expect(result.stderr).toBe("");
  expect(result.status).toBe(1);
  const oxlint = oxlintFindings(JSON.parse(result.stdout).diagnostics);
  expect(
    oxlint.map(({ file, line, message }) => [file.split("/").at(-1), line, message]).sort(),
  ).toEqual(
    eslint
      .flatMap(({ file, findings }) => findings.map(({ line, message }) => [file, line, message]))
      .sort(),
  );

  await expect(snapshotDocument({ eslint: snapshotCases(eslint), oxlint })).toMatchFileSnapshot(
    "snapshots/test-no-style-class-assertion.snap",
  );
});

it("exports the rule for opt-in use", () => {
  expect(plugin.rules[rule]).toBeDefined();
  expect(plugin.configs.recommended.rules[`no-mistakes/${rule}`]).toBeUndefined();
  expect(plugin.configs.strict.rules[`no-mistakes/${rule}`]).toBeUndefined();
});
