"use strict";

const { resolveVariable } = require("./postgres-query-text");
const { memberPropertyName } = require("./postgres-executor");
const { propertyText } = require("./postgres-no-executor-adapter-logic");
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

function patternKey(property) {
  if (property.computed) return propertyText({ computed: true, property: property.key });
  const key = property.key;
  if (key?.type === "Identifier") return key.name;
  if (key?.type === "Literal" && typeof key.value === "string") {
    return key.value;
  }
  return null;
}

// `import * as db` then `db.read` is the named export. A computed identifier
// key is not that export. The module source has to be the configured module.
function namespaceImport(identifier, context, options) {
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
  if (options.importSpecifier && declaration?.source?.value !== options.importSpecifier) {
    return false;
  }
  return true;
}

function namespaceExecutor(node, context, options) {
  if (node?.type !== "MemberExpression") {
    return null;
  }
  const name = propertyText(node);
  if (!name || !(options.executorNames ?? []).includes(name)) {
    return null;
  }
  let object = node.object;
  while (object && TS_ASSERTIONS.has(object.type)) object = object.expression;
  if (object?.type !== "Identifier") {
    return null;
  }
  if (!namespaceImport(object, context, options)) {
    return null;
  }
  return { name: context.sourceCode.getText(node), kind: "member", aliasable: false };
}

// `const { query } = tx` is `tx.query` when `tx` is a proven receiver.
// `const { read } = db` is the namespace export. A default, `let`, or a
// computed identifier key can bind something else.
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
  const property = declarator.id.properties.find(
    (item) => item.type === "Property" && item.value === def.name,
  );
  if (!property) {
    return null;
  }
  const key = patternKey(property);
  // The unmatched key returns from the else. A fallthrough is not counted.
  if (key) {
    return destructuredKey(identifier, declarator, key, context, bindings, options);
  } else {
    return null;
  }
}

function destructuredKey(identifier, declarator, key, context, bindings, options) {
  let object = declarator.init;
  while (object && TS_ASSERTIONS.has(object.type)) object = object.expression;
  if (object?.type !== "Identifier") {
    return null;
  }
  if (namespaceImport(object, context, options) && options.executorNames.includes(key)) {
    return { name: identifier.name, kind: "identifier", aliasable: true };
  }
  if (
    key === "query" &&
    provenQueryReceiver(object, context, bindings, options, new Set()) === true
  ) {
    return { name: identifier.name, kind: "identifier", aliasable: false };
  }
  return null;
}

module.exports = {
  destructuredExecutor,
  isProvenQueryMember,
  namespaceExecutor,
};
