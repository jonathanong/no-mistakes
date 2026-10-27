import { Linter } from "eslint";
import { lint, linterCwd, plugin } from "./helpers.mjs";
import {
  eslintFindings,
  relativeDiagnosticFile,
  snapshotOptions,
} from "./diagnostic-snapshot-format.mjs";
export {
  normalizeDiagnosticText,
  relativeDiagnosticFile,
  snapshotDocument,
  eslintFindings,
  oxlintFindings,
} from "./diagnostic-snapshot-format.mjs";

// messages() stores one option value, even when that value is an array.
function messageSetting(option) {
  if (option === undefined) return "error";
  return ["error", option];
}

// RuleTester `options` is the array spread after the severity.
function testerSetting(options) {
  if (options === undefined) return "error";
  return ["error", ...options];
}

export function pluginFindings(
  code,
  rule,
  option,
  filename = "fixture.jsx",
  globals = {},
  sourceType = "module",
) {
  return eslintFindings(
    lint(code, { [`no-mistakes/${rule}`]: messageSetting(option) }, filename, globals, sourceType),
    filename,
  );
}

export function configuredFindings(
  code,
  rules,
  filename = "fixture.jsx",
  globals = {},
  sourceType = "module",
) {
  return eslintFindings(lint(code, rules, filename, globals, sourceType), filename);
}

// RuleTester cases carry their own languageOptions. helpers.lint would switch
// TypeScript files onto the TS parser and change which branches run.
export function languageOptionsFindings({ code, rule, options, filename, languageOptions }) {
  const linter = new Linter({
    configType: "flat",
    cwd: filename ? linterCwd(filename) : process.cwd(),
  });
  return eslintFindings(
    linter.verify(
      code,
      {
        // `**/*` does not match RuleTester filenames. `**` is the pattern
        // RuleTester's default config uses so every filename is linted.
        files: ["**"],
        languageOptions,
        plugins: { "no-mistakes": plugin },
        rules: { [`no-mistakes/${rule}`]: testerSetting(options) },
      },
      filename,
    ),
    filename || "",
  );
}

export function snapshotCases(cases) {
  return [...cases]
    .map((item) => ({
      file: item.file ?? "fixture.jsx",
      findings: item.findings,
      name: item.name,
      options: item.options === undefined ? null : item.options,
      ...(item.rule ? { rule: item.rule } : {}),
      ...(item.sourceType ? { sourceType: item.sourceType } : {}),
    }))
    .sort(
      (left, right) =>
        String(left.rule ?? "").localeCompare(String(right.rule ?? "")) ||
        String(left.name).localeCompare(String(right.name)),
    );
}

// Source order, not alphabetical. Findings inside each row are already sorted.
export function snapshotSuite(rule, languageOptions, suite) {
  const rows = [];
  for (const kind of ["valid", "invalid"]) {
    for (const item of suite[kind] ?? []) {
      const code = typeof item === "string" ? item : item.code;
      const filename = typeof item === "object" && item.filename ? item.filename : undefined;
      const options = typeof item === "object" && item.options !== undefined ? item.options : null;
      const caseLanguage = {
        ...languageOptions,
        ...(typeof item === "object" ? item.languageOptions : undefined),
      };
      const findings = languageOptionsFindings({
        code,
        filename,
        languageOptions: caseLanguage,
        options: options ?? undefined,
        rule,
      });
      // RuleTester required this rule to report on invalid cases and stay
      // silent on valid ones. Parse errors have no ruleId, so they do not count.
      const reported = findings.filter((item) => item.ruleId === `no-mistakes/${rule}`);
      if (
        (kind === "invalid" && reported.length === 0) ||
        (kind === "valid" && reported.length > 0)
      ) {
        throw new Error(
          `${kind} ${rule} case ${filename ?? "<input>"} produced ${reported.length} diagnostics`,
        );
      }
      rows.push({
        file: filename ? relativeDiagnosticFile(filename, filename) : "<input>",
        findings,
        kind,
        options: snapshotOptions(options),
      });
    }
  }
  return rows;
}
