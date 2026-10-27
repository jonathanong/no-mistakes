import { tmpdir } from "node:os";
import { isAbsolute, relative } from "node:path";
import { __dirname } from "./helpers.mjs";
import { resolve } from "node:path";

const packageRoot = resolve(__dirname, "..").replaceAll("\\", "/");
const repoRoot = resolve(__dirname, "../../..").replaceAll("\\", "/");
const tempRoots = [tmpdir(), process.env.TEMP, process.env.TMP, process.env.TMPDIR]
  .filter(Boolean)
  .map((item) => item.replaceAll("\\", "/").replace(/\/$/, ""))
  .sort((left, right) => right.length - left.length);

// File snapshots keep fixture-relative paths and a fixed order. Absolute paths
// and diagnostic order would change across machines and worker counts.
export function normalizeDiagnosticText(text) {
  let value = String(text).replaceAll("\\", "/");
  value = value.replaceAll(packageRoot, "<root>");
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
  const fromPackage = relative(packageRoot, slash).replaceAll("\\", "/");
  if (fromPackage && !fromPackage.startsWith("..") && !isAbsolute(fromPackage)) return fromPackage;
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
      ...(item.fatal ? { fatal: true } : {}),
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

// Normalize real host paths without rewriting regex escapes in option strings.
export function snapshotOptions(options) {
  return JSON.parse(
    JSON.stringify(options, (_key, value) => {
      if (typeof value !== "string") return value;
      const slash = value.replaceAll("\\", "/");
      return [repoRoot, ...tempRoots].some(
        (prefix) => slash === prefix || slash.startsWith(`${prefix}/`),
      )
        ? normalizeDiagnosticText(value)
        : value;
    }),
  );
}
