"use strict";

function safeRegExp(source) {
  try {
    return new RegExp(source);
  } catch {
    return null;
  }
}

function patternToRegExp(pattern) {
  if (pattern.startsWith("/") && pattern.endsWith("/") && pattern.length > 2) {
    return safeRegExp(pattern.slice(1, -1));
  }
  let source = "^";
  for (let index = 0; index < pattern.length; index += 1) {
    const ch = pattern[index];
    const next = pattern[index + 1];
    if (ch === "*" && next === "*") {
      if (pattern[index + 2] === "/") {
        source += "(?:.*/)?";
        index += 2;
      } else {
        source += ".*";
        index += 1;
      }
    } else if (ch === "*") {
      source += "[^/]*";
    } else if (ch === "?") {
      source += "[^/]";
    } else {
      source += ch.replace(/[\\^$+?.()|[\]{}]/g, "\\$&");
    }
  }
  return safeRegExp(`${source}$`);
}

function compileTargets(options, optionKey) {
  return (options[optionKey] || [])
    .map((target) => ({
      sourceSpecifierPatterns: (target.sourceSpecifierPatterns || [])
        .map(patternToRegExp)
        .filter(Boolean),
      calleeNamePatterns: (target.calleeNamePatterns || []).map(patternToRegExp).filter(Boolean),
    }))
    .filter(
      (target) => target.sourceSpecifierPatterns.length > 0 && target.calleeNamePatterns.length > 0,
    );
}

function matchesAny(value, patterns) {
  return typeof value === "string" && patterns.some((pattern) => pattern.test(value));
}

function matchingTargets(targets, source, calleeName) {
  return targets.filter(
    (target) =>
      matchesAny(source, target.sourceSpecifierPatterns) &&
      matchesAny(calleeName, target.calleeNamePatterns),
  );
}

function targetMatches(targets, source, calleeName) {
  return matchingTargets(targets, source, calleeName).length > 0;
}

module.exports = {
  compileTargets,
  matchingTargets,
  matchesAny,
  patternToRegExp,
  targetMatches,
};
