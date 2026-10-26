"use strict";

const { patternToRegExp } = require("./async-patterns");
const { normalizeRequiredProperties } = require("./imported-call-options");
const { repoRelativeFilename, stringMatches } = require("./module-mock-helpers");

function compileSelectorPattern(pattern, globSource) {
  const regexForm = pattern.startsWith("/") && pattern.endsWith("/") && pattern.length > 2;
  if (regexForm) {
    try {
      return { regex: new RegExp(pattern.slice(1, -1)), glob: false };
    } catch (error) {
      throw new Error(`Invalid selector pattern ${pattern}: ${error.message}`);
    }
  }
  const regex = patternToRegExp(globSource);
  return regex ? { regex, glob: true } : null;
}

function compileNamePatterns(patterns) {
  if (!Array.isArray(patterns)) return [];
  return patterns
    .filter((pattern) => typeof pattern === "string" && pattern)
    .map((pattern) => compileSelectorPattern(pattern, pattern)?.regex)
    .filter(Boolean);
}

function globSegment(value) {
  return value.replaceAll("%", "%25").replaceAll("/", "%2F");
}

// Globs use `.` as a segment separator. `/regex/` is tested against the dotted path.
function compileDottedPattern(pattern) {
  if (typeof pattern !== "string" || !pattern) return null;
  return compileSelectorPattern(pattern, globSegment(pattern).replaceAll(".", "/"));
}

function compileDottedPatterns(patterns) {
  return patterns.map((pattern) => compileDottedPattern(pattern)).filter(Boolean);
}

function matchesDotted(path, patterns) {
  const segments = Array.isArray(path) ? path : path.split(".");
  const slashed = segments.map(globSegment).join("/");
  const dotted = segments.join(".");
  return patterns.some((pattern) => pattern.regex.test(pattern.glob ? slashed : dotted));
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
