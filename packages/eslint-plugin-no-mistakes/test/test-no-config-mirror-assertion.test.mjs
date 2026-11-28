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

const rule = "test-no-config-mirror-assertion";
const root = resolve(__dirname, "../../../test-cases/eslint-snippets/fixture", rule);
const defaultFiles = [
  "sources.test.ts",
  "sources-cjs.spec.cjs",
  "edge-cases.test.ts",
  "sinks.test.ts",
  "parsed.test.ts",
  "custom-parser.test.ts",
  "cross-file.test.ts",
  "mixed-module.test.ts",
  "executed.test.ts",
  "fixtures-and-written.test.ts",
  "custom-fixture-dirs.test.ts",
  "suppressions.test.ts",
  "suppressed-file.test.ts",
  "suppressed-file-no-reason.test.ts",
  "exemptions.test.ts",
  "exemptions-global-vi.test.ts",
  "shadowed.test.ts",
  "helper.ts",
];

it("snapshots config-mirror assertion diagnostics in ESLint and Oxlint", async () => {
  const eslintDefault = defaultFiles.map((file) => {
    const sourceType = file.endsWith(".cjs") ? "commonjs" : "module";
    return {
      name: file,
      file,
      rule,
      findings: pluginFindings(fixture(`${rule}/${file}`), rule, undefined, file, {}, sourceType),
      ...(sourceType === "commonjs" ? { sourceType } : {}),
    };
  });
  const optionCases = [
    {
      name: "custom-parser.test.ts (configured parsers)",
      file: "custom-parser.test.ts",
      rule,
      options: {
        parsers: [
          { module: "smol-toml", name: "parse" },
          { module: "custom-ini", name: "default" },
        ],
      },
      findings: pluginFindings(
        fixture(`${rule}/custom-parser.test.ts`),
        rule,
        {
          parsers: [
            { module: "smol-toml", name: "parse" },
            { module: "custom-ini", name: "default" },
          ],
        },
        "custom-parser.test.ts",
      ),
    },
    {
      name: "custom-fixture-dirs.test.ts (testdata)",
      file: "custom-fixture-dirs.test.ts",
      rule,
      options: { fixtureDirs: ["testdata"] },
      findings: pluginFindings(
        fixture(`${rule}/custom-fixture-dirs.test.ts`),
        rule,
        { fixtureDirs: ["testdata"] },
        "custom-fixture-dirs.test.ts",
      ),
    },
  ];

  const result = spawnSync(
    process.execPath,
    [
      resolve(__dirname, "../../../node_modules/oxlint/bin/oxlint"),
      "--config",
      resolve(root, ".oxlintrc.json"),
      "--format",
      "json",
      ...defaultFiles.map((file) => resolve(root, file)),
    ],
    { encoding: "utf8" },
  );
  expect(result.stderr).toBe("");
  expect(result.status).toBe(1);
  const oxlint = oxlintFindings(JSON.parse(result.stdout).diagnostics);
  expect(oxlint.map(({ file, line }) => [file.split("/").at(-1), line]).sort()).toEqual(
    eslintDefault.flatMap(({ file, findings }) => findings.map(({ line }) => [file, line])).sort(),
  );
  await expect(
    snapshotDocument({ eslint: snapshotCases([...eslintDefault, ...optionCases]), oxlint }),
  ).toMatchFileSnapshot("snapshots/test-no-config-mirror-assertion.snap");
});

it("exports the rule for opt-in use", () => {
  expect(plugin.rules[rule]).toBeDefined();
  expect(plugin.configs.recommended.rules[`no-mistakes/${rule}`]).toBeUndefined();
  expect(plugin.configs.strict.rules[`no-mistakes/${rule}`]).toBeUndefined();
});
