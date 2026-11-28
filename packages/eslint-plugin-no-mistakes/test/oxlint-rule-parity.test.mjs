import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { oxlintFindings, pluginFindings, snapshotDocument } from "./diagnostic-snapshot.mjs";
import { __dirname } from "./helpers.mjs";

// Fixture-relative line from each rule's saved ESLint case (#4189–#4265).
const expectedLine = {
  "async-try-catch-return-await": 13,
  "await-array-methods": 3,
  "forbidden-member-chains": 1,
  "gh-api-argument-safety": 4,
  "module-mock-boundary": 8,
  "module-mock-preserve-exports": 3,
  "nextjs-metadata-exports-location": 22,
  "nextjs-no-manual-script-tags": 4,
  "nextjs-static-fetch-method": 3,
  "nextjs-static-fetch-url": 1,
  "no-argv-main-guard": 4,
  "no-banned-import-outside-allowed-paths": 2,
  "no-comment-phrases": 1,
  "no-consecutive-unbatched-reads": 5,
  "no-delete-property": 1,
  "no-global-fetch-outside-helper": 1,
  "no-import-only-test-files": 1,
  "no-inline-noop-promise-catch": 1,
  "no-locale-less-date-format": 1,
  "no-placeholder-never-type-exports": 1,
  "no-top-level-await": 1,
  "no-vitest-sequential": 4,
  "playwright-assertion-timeout-cap": 23,
  "playwright-consistent-attribute": 1,
  "playwright-defaults": 1,
  "playwright-naming-convention": 1,
  "playwright-no-empty": 1,
  "playwright-no-hoisted-unique-token": 3,
  "playwright-no-raw-scroll-pagination": 2,
  "playwright-no-swallowed-visibility": 22,
  "playwright-prefer-get-by-test-id": 4,
  "playwright-require-exported-component-attribute": 2,
  "playwright-selector-priority": 4,
  "playwright-unique": 3,
  "postgres-cursor-call-contract": 2,
  "postgres-no-async-sql-builder": 3,
  "postgres-no-correlated-sql-builder": 6,
  "postgres-no-executor-adapter": 17,
  "postgres-no-manual-transaction": 6,
  "postgres-no-nonliteral-sql-registry-key": 7,
  "postgres-no-unbounded-query-fanout": 11,
  "postgres-pool-requires-error-listener": 7,
  "react-no-iife-in-jsx": 44,
  "react-no-nullish-react-node": 14,
  "react-no-use-promise-resolve": 3,
  "require-options-on-imported-call": 4,
  "require-top-level-registration": 1,
  "restricted-api-usage": 4,
  "server-require-nullable-fetch-wrapper": 11,
  "test-no-style-class-assertion": 1,
  "test-no-typed-enqueue-fake": 9,
  "test-no-wall-time-assertions": 1,
  "ts-no-const-aliases": 3,
  "ts-no-export-renaming": 5,
  "ts-no-function-aliases": 3,
  "ts-preserve-null-option-defaults": 7,
  "vitest-mock-test-file-naming": 1,
  "vitest-timeout-cap": 5,
};

const root = resolve(__dirname, "../../..");
const fixtures = resolve(root, "test-cases");
const configPath = "oxlint-rule-parity.json";
const { overrides } = JSON.parse(readFileSync(resolve(fixtures, configPath), "utf8"));

function lintWithOxlint(files) {
  const result = spawnSync(
    process.execPath,
    [
      resolve(root, "node_modules/oxlint/bin/oxlint"),
      "--config",
      configPath,
      "--format",
      "json",
      "--no-ignore",
      "--threads",
      "2",
      ...files,
    ],
    { cwd: fixtures, encoding: "utf8", timeout: 15_000 },
  );
  expect(result.error, result.stderr).toBeUndefined();
  expect(result.status, result.stdout + result.stderr).toBe(1);
  return oxlintFindings(JSON.parse(result.stdout).diagnostics);
}

it("snapshots saved rule diagnostics from ESLint and Oxlint", async () => {
  const oxlint = lintWithOxlint(overrides.flatMap(({ files }) => files));
  const sections = [];
  let findingCount = 0;
  for (const {
    files: [file],
    rules,
  } of overrides) {
    const source = readFileSync(resolve(fixtures, file), "utf8");
    for (const [id, setting] of Object.entries(rules)) {
      const rule = id.slice("no-mistakes/".length);
      const code = `no-mistakes(${rule})`;
      const option = Array.isArray(setting) ? setting[1] : undefined;
      const eslintFindings = pluginFindings(source, rule, option, `test-cases/${file}`);
      const oxlintFindingsForRule = oxlint.filter(
        (finding) => finding.file === file && finding.code === code,
      );

      expect(eslintFindings.length, id).toBeGreaterThan(0);
      expect(
        eslintFindings.map((finding) => finding.ruleId),
        file,
      ).toEqual(eslintFindings.map(() => id));
      expect(
        oxlintFindingsForRule.map(({ line, message }) => [line, message]),
        id,
      ).toEqual(eslintFindings.map(({ line, message }) => [line, message]));
      expect(expectedLine[rule], rule).toEqual(expect.any(Number));
      expect(
        oxlintFindingsForRule.some((finding) => finding.line === expectedLine[rule]),
        `${rule} line ${expectedLine[rule]}`,
      ).toBe(true);
      await expect(snapshotDocument(oxlintFindingsForRule)).toMatchFileSnapshot(
        `snapshots/oxlint-diagnostics/${rule}.snap`,
      );

      findingCount += oxlintFindingsForRule.length;
      sections.push(
        [
          `## ${rule}`,
          file,
          ...eslintFindings.map(
            ({ line, column, ruleId, messageId, message }) =>
              `ESLint ${line}:${column} ${ruleId} (${messageId}) ${message}`,
          ),
          ...oxlintFindingsForRule.map(
            ({ line, column, code, message }) => `Oxlint ${line}:${column} ${code} ${message}`,
          ),
        ].join("\n"),
      );
    }
  }

  expect(oxlint).toHaveLength(findingCount);
  await expect(`${sections.sort().join("\n\n")}\n`).toMatchFileSnapshot(
    "snapshots/oxlint-rule-parity.snap",
  );
});
