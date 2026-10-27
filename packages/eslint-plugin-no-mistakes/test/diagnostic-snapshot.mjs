import { tmpdir } from "node:os";
import { isAbsolute, relative, resolve } from "node:path";
import { Linter } from "eslint";
import { __dirname, lint, linterCwd, plugin } from "./helpers.mjs";

const repoRoot = resolve(__dirname, "../../..").replaceAll("\\", "/");
const tempRoots = [tmpdir(), process.env.TEMP, process.env.TMP, process.env.TMPDIR]
  .filter(Boolean)
  .map((item) => item.replaceAll("\\", "/").replace(/\/$/, ""))
  .sort((left, right) => right.length - left.length);

// File snapshots keep fixture-relative paths and a fixed order. Absolute paths
// and diagnostic order would change across machines and worker counts.
export function normalizeDiagnosticText(text) {
  let value = String(text).replaceAll("\\", "/");
  if (repoRoot) value = value.replaceAll(repoRoot, "<root>");
  for (const prefix of tempRoots) value = value.replaceAll(prefix, "<temp>");
  return (
    value
      .replace(
        /(?<![A-Za-z0-9_>])(?:[A-Za-z]:)?(?:\/private\/var\/folders|\/var\/folders)\/[^/\s]+\/[^/\s]+\/T(?:\/\S*)?/gi,
        "<temp>",
      )
      .replace(
        /(?<![A-Za-z0-9_>])(?:[A-Za-z]:)?\/Users\/[^/\s]+\/AppData\/Local\/Temp(?:\/\S*)?/gi,
        "<temp>",
      )
      .replace(/(?<![A-Za-z0-9_>])(?:\/private\/tmp|\/var\/tmp|\/tmp)(?:\/\S*)?/gi, "<temp>")
      .replace(/\b(pid|ppid)\s*[:=]\s*\d+\b/gi, "$1 <pid>")
      // Keep "30000 ms" cap text. Only profiler timings such as "85ms" are unstable.
      .replace(/\b\d+(?:\.\d+)?(?:ms|us|µs|ns)\b/gi, "<timing>")
  );
}

export function relativeDiagnosticFile(filename, fallback = "") {
  const fallbackSlash = String(fallback).replaceAll("\\", "/");
  if (!filename) return fallbackSlash;
  const slash = String(filename).replaceAll("\\", "/");
  // Keep the relative case name. An absolute fallback is a resolved path.
  const sameName = slash === fallbackSlash || slash.endsWith(`/${fallbackSlash}`);
  if (fallbackSlash && !isAbsolute(fallbackSlash) && sameName) return fallbackSlash;
  if (!isAbsolute(slash)) return slash;
  const fromRepo = relative(repoRoot, slash).replaceAll("\\", "/");
  if (fromRepo !== "" && !fromRepo.startsWith("..") && !isAbsolute(fromRepo)) return fromRepo;
  for (const prefix of tempRoots) {
    if (slash === prefix || slash.startsWith(`${prefix}/`)) {
      const rest = slash.slice(prefix.length).replace(/^\//, "");
      const kept = rest.split("/").slice(1).join("/");
      return `<temp>/${kept || rest.split("/").pop() || ""}`;
    }
  }
  return normalizeDiagnosticText(slash);
}

function compareDiagnostics(left, right) {
  return (
    String(left.file).localeCompare(String(right.file)) ||
    (left.line ?? 0) - (right.line ?? 0) ||
    (left.column ?? 0) - (right.column ?? 0) ||
    String(left.messageId || left.code || "").localeCompare(
      String(right.messageId || right.code || ""),
    ) ||
    String(left.message || "").localeCompare(String(right.message || ""))
  );
}

function sortValue(value) {
  if (Array.isArray(value)) return value.map(sortValue);
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.keys(value)
        .sort()
        .map((key) => [key, sortValue(value[key])]),
    );
  }
  return value;
}

export function snapshotDocument(value) {
  return `${JSON.stringify(sortValue(value), null, 2)}\n`;
}

export function eslintFindings(messages, fallbackFile = "") {
  return messages
    .map((item) => ({
      column: item.column,
      endColumn: item.endColumn ?? null,
      endLine: item.endLine ?? null,
      file: relativeDiagnosticFile(item.filename || fallbackFile, fallbackFile),
      line: item.line,
      message: normalizeDiagnosticText(item.message ?? ""),
      messageId: item.messageId ?? null,
      ruleId: item.ruleId ?? null,
      severity: item.severity,
    }))
    .sort(compareDiagnostics);
}

export function oxlintFindings(diagnostics, fallbackFile = "") {
  return diagnostics
    .map((item) => ({
      code: item.code ?? null,
      column: item.labels?.[0]?.span?.column ?? null,
      file: relativeDiagnosticFile(item.filename, fallbackFile),
      line: item.labels?.[0]?.span?.line ?? null,
      message: normalizeDiagnosticText(item.message ?? ""),
    }))
    .sort(compareDiagnostics);
}

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
        options: JSON.parse(normalizeDiagnosticText(JSON.stringify(options))),
      });
    }
  }
  return rows;
}
