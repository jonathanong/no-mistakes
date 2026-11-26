"use strict";

const { resolveVariable } = require("./postgres-query-text");
const { memberPropertyName } = require("./postgres-executor");
const { propertyText } = require("./postgres-no-executor-adapter-logic");
const {
  findBindingPath,
  projectPath,
  resolveNamespaceForKey,
} = require("./postgres-no-executor-adapter-namespace");
const {
  TS_ASSERTIONS,
  constInitializer,
  variableIsImportedExecutor,
  variableIsScopedExecutor,
} = require("./postgres-no-executor-adapter-binding");

function broadQueryMembers(options) {
  return !options.importSpecifier && (options.executorNames ?? []).includes("query");
}

// A configured module judges `.query` only on a proven receiver: a configured
// import, a scoped factory or typed parameter, or a const alias of one.
// `executorNames: [query]` with no module is the explicit broad selector.
function provenQueryReceiver(node, context, bindings, options, seen) {
  let current = node;
  while (current && TS_ASSERTIONS.has(current.type)) current = current.expression;
  if (!current || current.type === "ChainExpression" || current.optional) return null;
  if (current.type === "ConditionalExpression") {
    const consequent = provenQueryReceiver(current.consequent, context, bindings, options, seen);
    const alternate = provenQueryReceiver(current.alternate, context, bindings, options, seen);
    return consequent && alternate ? true : null;
  }
  if (current.type !== "Identifier") return null;
  const variable = resolveVariable(current, context);
  if (
    variableIsImportedExecutor(variable, options) ||
    variableIsScopedExecutor(variable, bindings.scoped ?? [], current)
  ) {
    return true;
  }
  const init = constInitializer(current, context, seen);
  return init ? provenQueryReceiver(init, context, bindings, options, seen) : null;
}

function isProvenQueryMember(node, context, bindings, options) {
  if (node?.type !== "MemberExpression") return false;
  if (memberPropertyName(node) !== "query") return false;
  if (broadQueryMembers(options)) return true;
  if (!options.importSpecifier || bindings.queryMembers !== true) return false;
  return provenQueryReceiver(node.object, context, bindings, options, new Set()) === true;
}

// `import * as db` then `db.read` is the named export. A non-const computed
// identifier key is not that export. The module source has to be the configured module.
function namespaceImport(identifier, context, importSpecifier) {
  const variable = resolveVariable(identifier, context);
  if (!variable || variable.defs.length !== 1) {
    return false;
  }
  const def = variable.defs[0];
  if (def.type !== "ImportBinding" || def.node?.type !== "ImportNamespaceSpecifier") {
    return false;
  }
  const declaration = def.parent;
  if (declaration?.importKind === "type") {
    return false;
  }
  if (importSpecifier && declaration?.source?.value !== importSpecifier) {
    return false;
  }
  return true;
}

function namespaceExecutor(node, context, options) {
  if (node?.type !== "MemberExpression") {
    return null;
  }
  const name = propertyText(node, context);
  if (!name || !(options.executorNames ?? []).includes(name)) {
    return null;
  }
  if (
    !resolveNamespaceForKey(
      node.object,
      name,
      context,
      options.importSpecifier,
      namespaceImport,
      new Set(),
    )
  ) {
    return null;
  }
  return { name: context.sourceCode.getText(node), kind: "member", aliasable: false };
}

// `const { query } = tx` is `tx.query` when `tx` is a proven receiver.
// `const { read } = db` (or a nested/wrapped namespace destructure) is the
// namespace export. A default, `let`, or non-const computed key can bind something else.
function destructuredExecutor(identifier, context, bindings, options) {
  const variable = resolveVariable(identifier, context);
  if (!variable || variable.defs.length !== 1) {
    return null;
  }
  const def = variable.defs[0];
  if (def.type !== "Variable" || def.parent?.kind !== "const") {
    return null;
  }
  const declarator = def.node;
  if (declarator?.id?.type !== "ObjectPattern") {
    return null;
  }
  const path = findBindingPath(declarator.id, def.name, context);
  const last = path?.at(-1);
  if (last?.kind !== "prop") {
    return null;
  }
  const owner = projectPath(
    declarator.init,
    path.slice(0, -1),
    last.key,
    context,
    new Set([variable]),
  );
  return owner ? destructuredKey(identifier, owner, last.key, context, bindings, options) : null;
}

function destructuredKey(identifier, owner, key, context, bindings, options) {
  if (
    options.executorNames.includes(key) &&
    resolveNamespaceForKey(owner, key, context, options.importSpecifier, namespaceImport, new Set())
  ) {
    return { name: identifier.name, kind: "identifier", aliasable: true };
  }
  if (
    key === "query" &&
    provenQueryReceiver(owner, context, bindings, options, new Set()) === true
  ) {
    return { name: identifier.name, kind: "identifier", aliasable: false };
  }
  return null;
}

module.exports = {
  destructuredExecutor,
  isProvenQueryMember,
  namespaceExecutor,
  namespaceImport,
};
