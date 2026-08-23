"use strict";

const { stringMatches } = require("./module-mock-helpers");
const { normalizeFilename } = require("./postgres-cursor-ast");

const DEFAULT_CURSOR_INCLUDE = ["**/*.{ts,mts,tsx,js,mjs}"];
const DEFAULT_CURSOR_ANNOTATION = "^\\s*/\\*\\s*\\S[^]*?\\*/";
const DEFAULT_SQL_TAG_MODULES = ["sql-template-strings"];

function expandBraces(pattern) {
  const start = pattern.indexOf("{");
  const end = pattern.indexOf("}", start + 1);
  if (start === -1 || end === -1) return [pattern];
  const prefix = pattern.slice(0, start);
  const suffix = pattern.slice(end + 1);
  return pattern
    .slice(start + 1, end)
    .split(",")
    .flatMap((alt) => expandBraces(`${prefix}${alt}${suffix}`));
}

function globListMatches(filename, patterns) {
  return patterns.some((pattern) => stringMatches(filename, expandBraces(pattern)));
}

function stringArray(value) {
  if (value === undefined) return undefined;
  if (!Array.isArray(value) || value.some((entry) => typeof entry !== "string")) return undefined;
  return value;
}

function optionalStringArray(value, present) {
  if (!present) return undefined;
  return Array.isArray(value) && value.every((entry) => typeof entry === "string") ? value : null;
}

function resolveCursorContractOptions(raw) {
  if (raw === null || typeof raw !== "object" || Array.isArray(raw)) return null;
  const modules = stringArray(raw.modules);
  const executors = stringArray(raw.executors);
  if (!modules?.length || !executors?.length) return null;
  if (raw.annotation !== undefined && typeof raw.annotation !== "string") return null;
  const include = optionalStringArray(raw.include, raw.include !== undefined);
  const exclude = optionalStringArray(raw.exclude, raw.exclude !== undefined);
  const includeFiles = optionalStringArray(raw.includeFiles, raw.includeFiles !== undefined);
  const sqlTagModules = optionalStringArray(raw.sqlTagModules, raw.sqlTagModules !== undefined);
  if (include === null || exclude === null || includeFiles === null || sqlTagModules === null) {
    return null;
  }
  let annotation;
  try {
    annotation = new RegExp(
      typeof raw.annotation === "string" ? raw.annotation : DEFAULT_CURSOR_ANNOTATION,
    );
  } catch {
    throw new Error(
      `postgres-cursor-call-contract annotation is not a valid regular expression: ${String(raw.annotation)}`,
    );
  }
  return {
    modules: new Set(modules),
    executors: new Set(executors),
    include: include ?? DEFAULT_CURSOR_INCLUDE,
    exclude: exclude ?? [],
    includeFiles: (includeFiles ?? []).map((file) => file.replace(/^(?:\.\/)+/, "")),
    sqlTagModules: new Set(sqlTagModules ?? DEFAULT_SQL_TAG_MODULES),
    annotation,
  };
}

function matchesCursorFile(context, options) {
  const filename = normalizeFilename(context).replace(/^(?:\.\/)+/, "");
  if (options.includeFiles.includes(filename)) return true;
  if (options.exclude.length > 0 && globListMatches(filename, options.exclude)) return false;
  return globListMatches(filename, options.include);
}

module.exports = {
  DEFAULT_CURSOR_INCLUDE,
  DEFAULT_SQL_TAG_MODULES,
  matchesCursorFile,
  resolveCursorContractOptions,
};
