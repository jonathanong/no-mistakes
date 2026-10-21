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
  if (node.type === "Literal") return String(node.value);
  if (typeof node.name === "string") return node.name;
  return literalString(node);
}

function memberPropertyName(node) {
  let current = node;
  while (current?.type === "ChainExpression") current = current.expression;
  if (current?.type !== "MemberExpression") return null;
  if (current.computed && current.property?.type === "Identifier") return null;
  return propertyName(current.property);
}

// The call that actually runs, after optional chaining, a comma callee, and
// `.call` / a literal `.apply` array. Opaque `.apply` or a spread `.call`
// returns null so callers do not read the wrong argument.
function logicalCall(node) {
  if (node?.type !== "CallExpression") return null;
  let callee = node.callee;
  let args = node.arguments;
  for (let depth = 0; depth < 4; depth += 1) {
    while (callee?.type === "ChainExpression") callee = callee.expression;
    if (callee?.type === "SequenceExpression" && callee.expressions.length > 0) {
      callee = callee.expressions[callee.expressions.length - 1];
      continue;
    }
    if (callee?.type !== "MemberExpression") break;
    const name = memberPropertyName(callee);
    if (name !== "call" && name !== "apply") break;
    if (name === "call") {
      if (args.some((arg) => arg?.type === "SpreadElement")) return null;
      callee = callee.object;
      args = args.slice(1);
      continue;
    }
    const list = args[1]?.type === "ChainExpression" ? args[1].expression : args[1];
    if (
      list?.type !== "ArrayExpression" ||
      list.elements.some((element) => !element || element.type === "SpreadElement")
    ) {
      return null;
    }
    callee = callee.object;
    args = list.elements;
  }
  while (callee?.type === "ChainExpression") callee = callee.expression;
  return { callee, arguments: args };
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

function unwrapCallee(callee) {
  let current = callee;
  for (let depth = 0; depth < 4; depth += 1) {
    if (current?.type === "ChainExpression") {
      current = current.expression;
      continue;
    }
    if (current?.type === "SequenceExpression" && current.expressions.length > 0) {
      current = current.expressions[current.expressions.length - 1];
      continue;
    }
    break;
  }
  return current;
}

function isModuleMockMemberCall(node, context) {
  const callee = unwrapCallee(node.callee);
  if (callee?.type !== "MemberExpression") return false;
  const method = memberPropertyName(callee);
  if (!MODULE_MOCK_METHODS.has(method)) return false;
  if (!isFrameworkBinding(callee.object, context)) return false;
  return {
    framework: frameworkBindingModule(callee.object, context),
    method,
    namespace: expressionName(callee.object),
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
  logicalCall,
  moduleMockSpecifierArgument,
  pathAllowed,
  propertyName,
  repoRelativeFilename,
  stringMatches,
};
