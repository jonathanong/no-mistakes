"use strict";

const { patternToRegExp } = require("./async-patterns");
const { normalizeRequiredProperties } = require("./imported-call-options");
const { repoRelativeFilename, stringMatches } = require("./module-mock-helpers");

function compileNamePatterns(patterns) {
  if (!Array.isArray(patterns)) return [];
  return patterns
    .filter((pattern) => typeof pattern === "string" && pattern)
    .map((pattern) => patternToRegExp(pattern))
    .filter(Boolean);
}

// Globs use `.` as a segment separator. `/regex/` is tested against the dotted path.
function compileDottedPattern(pattern) {
  if (typeof pattern !== "string" || !pattern) return null;
  const regexForm = pattern.startsWith("/") && pattern.endsWith("/") && pattern.length > 2;
  const regex = patternToRegExp(regexForm ? pattern : pattern.replaceAll(".", "/"));
  return regex ? { regex, glob: !regexForm } : null;
}

function compileDottedPatterns(patterns) {
  return patterns.map((pattern) => compileDottedPattern(pattern)).filter(Boolean);
}

function matchesDotted(path, patterns) {
  const slashed = path.replaceAll(".", "/");
  return patterns.some((pattern) => pattern.regex.test(pattern.glob ? slashed : path));
}

function selectorTarget(target, requiredProperties) {
  const compiled = {
    optionsPosition: target.optionsPosition,
    requiredProperties,
    propertyMatch: target.propertyMatch === "all" ? "all" : "any",
  };
  if (typeof target.message === "string" && target.message) compiled.message = target.message;
  return compiled;
}

function compileCallSelectors(options) {
  const globals = [];
  const members = [];
  for (const target of options.targets || []) {
    // Import targets keep today's matcher. Global and receiver kinds cannot share a target.
    if (!target || Array.isArray(target.sourceSpecifierPatterns)) continue;
    const hasGlobal = Array.isArray(target.globalNamePatterns);
    const hasReceiver = Array.isArray(target.receiverPatterns);
    const optionsPosition = target.optionsPosition;
    const names = normalizeRequiredProperties(target.requiredProperties);
    if (
      hasGlobal === hasReceiver ||
      !Number.isInteger(optionsPosition) ||
      optionsPosition < 1 ||
      names.length === 0
    ) {
      continue;
    }
    const base = selectorTarget(target, names);
    if (hasGlobal) {
      const globalNamePatterns = compileNamePatterns(target.globalNamePatterns);
      if (globalNamePatterns.length === 0) continue;
      globals.push({ ...base, globalNamePatterns });
      continue;
    }
    const receiverPatterns = compileDottedPatterns(target.receiverPatterns);
    const calleeNamePatterns = compileNamePatterns(target.calleeNamePatterns);
    if (receiverPatterns.length === 0 || calleeNamePatterns.length === 0) continue;
    members.push({ ...base, receiverPatterns, calleeNamePatterns });
  }
  return { globals, members };
}

function shouldCheckOptionsFile(filename, options) {
  const checked = options.checkedPathPatterns || [];
  const allowed = options.allowedPathPatterns || [];
  if (checked.length === 0 && allowed.length === 0) return true;
  const file = repoRelativeFilename(filename || "");
  if (checked.length > 0 && !stringMatches(file, checked)) return false;
  return !stringMatches(file, allowed);
}

module.exports = {
  compileCallSelectors,
  matchesDotted,
  shouldCheckOptionsFile,
};
