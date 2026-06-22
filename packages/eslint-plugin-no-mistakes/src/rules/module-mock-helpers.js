"use strict";

const { collectPatternNames } = require("./ast-pattern-names");
const {
  expressionName,
  frameworkBindingModule,
  isFrameworkBinding,
} = require("./module-mock-framework");

const MODULE_MOCK_METHODS = new Set([
  "doMock",
  "doUnmock",
  "importMock",
  "mock",
  "setMock",
  "unmock",
  "unstable_mockModule",
]);
const PRESERVE_METHODS = new Set(["mock", "doMock", "unstable_mockModule"]);
const DEFAULT_INTERNAL_SPECIFIERS = ["./**", "../**", "/**"];

function propertyName(node) {
  if (!node) return null;
  return node.type === "Literal" ? String(node.value) : node.name;
}

function memberPropertyName(node) {
  if (!node?.computed) return propertyName(node?.property);
  return node.property?.type === "Literal" ? String(node.property.value) : null;
}

function literalString(node) {
  if (!node) return null;
  if (node.type === "Literal" && typeof node.value === "string") return node.value;
  if (node.type === "TemplateLiteral" && node.expressions.length === 0) {
    return node.quasis.map((quasi) => quasi.value.cooked ?? quasi.value.raw).join("");
  }
  return null;
}

function normalizeFilename(filename) {
  return filename.replace(/\\/g, "/");
}

function repoRelativeFilename(filename) {
  const cwd = normalizeFilename(process.cwd());
  const normalized = normalizeFilename(filename);
  return normalized.startsWith(`${cwd}/`) ? normalized.slice(cwd.length + 1) : normalized;
}

function globToRegExp(pattern) {
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
  source += "$";
  return new RegExp(source);
}

function safeRegExp(source) {
  try {
    return new RegExp(source);
  } catch {
    return null;
  }
}

function stringMatches(value, patterns) {
  return patterns.some((pattern) => {
    if (pattern.startsWith("/") && pattern.endsWith("/") && pattern.length > 2) {
      const regex = safeRegExp(pattern.slice(1, -1));
      return regex ? regex.test(value) : false;
    }
    return globToRegExp(pattern).test(value);
  });
}

function pathAllowed(filename, options) {
  const file = repoRelativeFilename(filename);
  const include = options.includePathPatterns ?? [];
  const exclude = options.excludePathPatterns ?? [];
  return (include.length === 0 || stringMatches(file, include)) && !stringMatches(file, exclude);
}

function isInternalSpecifier(specifier, options) {
  return stringMatches(specifier, options.internalSpecifiers ?? DEFAULT_INTERNAL_SPECIFIERS);
}

function moduleMockSpecifierArgument(node) {
  const direct = literalString(node);
  if (direct !== null) return { dynamic: false, specifier: direct };
  if (node?.type === "ImportExpression") {
    const specifier = literalString(node.source);
    return specifier === null ? { dynamic: true } : { dynamic: false, specifier };
  }
  if (node?.type === "CallExpression" && node.callee.type === "Import") {
    const specifier = literalString(node.arguments[0]);
    return specifier === null ? { dynamic: true } : { dynamic: false, specifier };
  }
  return { dynamic: true };
}

function isModuleMockMemberCall(node, context) {
  if (node.callee.type !== "MemberExpression") return false;
  const method = memberPropertyName(node.callee);
  if (!MODULE_MOCK_METHODS.has(method)) return false;
  if (!isFrameworkBinding(node.callee.object, context)) return false;
  return {
    framework: frameworkBindingModule(node.callee.object, context),
    method,
    namespace: expressionName(node.callee.object),
  };
}

function isPreserveMockCall(node, context) {
  const mock = isModuleMockMemberCall(node, context);
  return mock && PRESERVE_METHODS.has(mock.method) ? mock : false;
}

function importSpecifierName(specifier) {
  const imported = specifier.imported;
  if (!imported) return null;
  return imported.type === "Literal" ? String(imported.value) : imported.name;
}

module.exports = {
  collectPatternNames,
  expressionName,
  frameworkBindingModule,
  importSpecifierName,
  isFrameworkBinding,
  isInternalSpecifier,
  isModuleMockMemberCall,
  memberPropertyName,
  isPreserveMockCall,
  literalString,
  moduleMockSpecifierArgument,
  pathAllowed,
  propertyName,
  repoRelativeFilename,
  stringMatches,
};
