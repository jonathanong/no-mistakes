import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { expect, it } from "vitest";
import { oxlintFindings, pluginFindings } from "./diagnostic-snapshot.mjs";
import { __dirname } from "./helpers.mjs";

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
