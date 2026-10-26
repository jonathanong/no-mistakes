"use strict";

const { literalString } = require("../helpers");

function patternKey(property) {
  const key = property.key;
  if (property.computed && key.type === "Identifier") return null;
  if (!property.computed && typeof key.name === "string") return key.name;
  return literalString(key);
}

function variableFor(node, context) {
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables.find((item) => item.name === node.name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function boundName(pattern) {
  const node = pattern.type === "AssignmentPattern" ? pattern.left : pattern;
  return node.type === "Identifier" ? node.name : null;
}

function objectPatternKey(pattern, name) {
  for (const item of pattern.properties) {
    if (item.type !== "Property") continue;
    if (boundName(item.value) === name) return patternKey(item);
  }
  return null;
}

// `const` is one write. `let` and `var` follow only when every write is the
// initializer; a later assignment fails closed.
function isSingleWrite(def, variable) {
  const kind = def.parent?.kind;
  if (kind === "const") return true;
  if (kind !== "let" && kind !== "var") return false;
  return !variable.references.some((reference) => reference.isWrite() && !reference.init);
}

function resolveAlias(identifier, context, seen) {
  const variable = variableFor(identifier, context);
  if (!variable || seen.has(variable) || variable.defs?.length !== 1) return null;
  const def = variable.defs[0];
  if (def.type !== "Variable" || !def.node?.init || !isSingleWrite(def, variable)) return null;
  seen.add(variable);
  if (def.node.id.type === "ObjectPattern") {
    return {
      kind: "key",
      key: objectPatternKey(def.node.id, identifier.name),
      declarator: def.node,
    };
  }
  if (def.node.id.type !== "Identifier") return null;
  return { kind: "init", init: def.node.init, declarator: def.node };
}

module.exports = {
  patternKey,
  resolveAlias,
  variableFor,
};
