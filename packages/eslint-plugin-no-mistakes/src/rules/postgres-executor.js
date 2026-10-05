"use strict";

const { quasiText, sqlText, unwrapTs } = require("./postgres-query-text");
const { collectScopedExecutors, isScopedExecutor } = require("./postgres-scoped-executors");

const DEFAULT_IMPORT_SPECIFIER = "";
const DEFAULT_EXECUTOR_NAMES = ["query", "read", "write"];
const DEFAULT_CHUNK_FUNCTION_NAMES = ["chunkArray"];
const TRANSACTION_IMPORTS = new Set(["withTransaction", "withTransactionOptions"]);
const QUERY_PROPERTY = "query";
const TRANSACTION_COMMAND = /^\s*(?:BEGIN|COMMIT|ROLLBACK)\b/i;

function executorOptionDefaults(options = {}) {
  if (!options.importSpecifier && !Array.isArray(options.executorNames)) {
    throw new Error(
      "PostgreSQL option importSpecifier: set importSpecifier (or executorNames) to select executor calls; set executorNames: [] to select no executor calls (see docs/migrations/explicit-postgres-executors.md)",
    );
  }
  return {
    importSpecifier: options.importSpecifier ?? DEFAULT_IMPORT_SPECIFIER,
    executorNames: options.executorNames ?? (options.importSpecifier ? DEFAULT_EXECUTOR_NAMES : []),
    executorFactoryNames: options.executorFactoryNames ?? [],
    executorTypeNames: options.executorTypeNames ?? [],
    owners: options.owners ?? [],
    chunkFunctionNames: options.chunkFunctionNames ?? DEFAULT_CHUNK_FUNCTION_NAMES,
  };
}

function executorOptionSchema(extraProperties = {}) {
  return {
    type: "object",
    properties: {
      importSpecifier: { type: "string" },
      executorNames: { type: "array", items: { type: "string" } },
      executorFactoryNames: { type: "array", items: { type: "string" } },
      executorTypeNames: { type: "array", items: { type: "string" } },
      ...extraProperties,
    },
    additionalProperties: false,
  };
}

function importedName(specifier) {
  const imported = specifier?.imported;
  if (!imported) return null;
  return imported.type === "Literal" ? String(imported.value) : imported.name;
}

function executorBindings(program, options = {}) {
  const bindings = new Set();
  const defaults = executorOptionDefaults(options);
  const { importSpecifier, executorNames } = defaults;
  bindings.scoped = collectScopedExecutors(program, defaults);
  bindings.queryMembers = Boolean(importSpecifier) || executorNames.includes(QUERY_PROPERTY);
  for (const statement of program?.body ?? []) {
    if (statement.type !== "ImportDeclaration") continue;
    if (statement.importKind === "type") continue;
    if (importSpecifier && statement.source?.value !== importSpecifier) continue;
    for (const specifier of statement.specifiers ?? []) {
      if (specifier.type !== "ImportSpecifier") continue;
      if (specifier.importKind === "type") continue;
      const imported = importedName(specifier);
      if (importSpecifier && TRANSACTION_IMPORTS.has(imported)) bindings.add(QUERY_PROPERTY);
      if (imported && executorNames.includes(imported)) bindings.add(specifier.local.name);
    }
  }
  return bindings;
}

function staticQueryKey(node) {
  node = unwrapTs(node);
  if (!node) return false;
  if (node.type === "Literal") return node.value === QUERY_PROPERTY;
  if (node.type === "TemplateLiteral" && node.expressions.length === 0) {
    return quasiText(node.quasis[0]) === QUERY_PROPERTY;
  }
  return false;
}

function memberPropertyName(node) {
  if (!node || node.type !== "MemberExpression") return null;
  if (!node.computed) return node.property?.name ?? null;
  if (staticQueryKey(node.property)) return QUERY_PROPERTY;
  return sqlText(node.property);
}

function calleeName(call, bindings) {
  const callee = unwrapTs(call?.callee);
  if (!callee) return null;
  if (
    callee.type === "Identifier" &&
    (bindings?.has(callee.name) || isScopedExecutor(bindings?.scoped, callee.name, call))
  ) {
    return callee.name;
  }
  if (
    bindings?.queryMembers !== false &&
    callee.type === "MemberExpression" &&
    memberPropertyName(callee) === QUERY_PROPERTY
  ) {
    return QUERY_PROPERTY;
  }
  return null;
}

function isDatabaseCall(call, bindings) {
  return call?.type === "CallExpression" && calleeName(call, bindings) != null;
}

function firstCallArgument(call) {
  const argument = call?.arguments?.[0];
  if (!argument || argument.type === "SpreadElement") return null;
  return argument;
}

function stripSqlComments(text) {
  return text.replace(/\/\*[\s\S]*?\*\//g, " ").replace(/--[^\n]*/g, " ");
}

function isManualTransactionText(text) {
  return typeof text === "string" && TRANSACTION_COMMAND.test(stripSqlComments(text));
}

module.exports = {
  DEFAULT_CHUNK_FUNCTION_NAMES,
  DEFAULT_EXECUTOR_NAMES,
  DEFAULT_IMPORT_SPECIFIER,
  QUERY_PROPERTY,
  TRANSACTION_COMMAND,
  TRANSACTION_IMPORTS,
  calleeName,
  executorBindings,
  executorOptionDefaults,
  executorOptionSchema,
  firstCallArgument,
  isDatabaseCall,
  isManualTransactionText,
  memberPropertyName,
};
