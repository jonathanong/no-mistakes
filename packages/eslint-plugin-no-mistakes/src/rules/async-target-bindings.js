"use strict";

const { unwrapExpression } = require("./async-ast");
const { propertyName, literalString } = require("./module-mock-helpers");

const UNSTABLE_BINDING_DEF_TYPES = new Set(["Parameter", "CatchClause", "FunctionName"]);

function findVariable(scope, name) {
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function resolveVariable(node, context) {
  return findVariable(context.sourceCode.getScope(node), node.name);
}

// Erased declarations cannot shadow runtime globals such as Function or Map.
function resolveRuntimeVariable(node, context) {
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables.find((item) => item.name === node.name);
    if (
      variable?.defs.some(
        (definition) =>
          !["Type", "TypeParameter"].includes(definition.type) &&
          !(
            definition.type === "ImportBinding" &&
            (definition.node.importKind === "type" || definition.parent.importKind === "type")
          ),
      )
    )
      return variable;
    scope = scope.upper;
  }
  return null;
}

function isLocalRequire(id, context) {
  const variable = resolveVariable(id, context);
  return Boolean(variable?.defs.some((def) => def.type && def.type !== "ImplicitGlobalVariable"));
}

function isReassigned(id, context) {
  const variable = resolveVariable(id, context);
  if (!variable) return false;
  const writes = variable.references.filter((reference) => reference.isWrite());
  const initializationSites = new Set(
    writes.filter((reference) => reference.init).map((reference) => reference.identifier),
  );
  const isVar = variable.defs.some(
    (definition) => definition.type === "Variable" && definition.parent?.kind === "var",
  );
  return (
    writes.some((reference) => !reference.init) ||
    (isVar && initializationSites.size > 1) ||
    variable.defs.some((definition) => UNSTABLE_BINDING_DEF_TYPES.has(definition.type))
  );
}

function bindingIdentifier(node) {
  if (node?.type === "Identifier") return node;
  return node?.type === "AssignmentPattern" && node.left.type === "Identifier" ? node.left : null;
}

function staticComputedPropertyName(node) {
  return literalString(unwrapExpression(node));
}

function memberPropertyName(node) {
  if (!node.computed) return propertyName(node.property);
  return staticComputedPropertyName(node.property);
}

function recordObjectPatternBindings(pattern, source, recordDirect) {
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const name = property.computed
      ? staticComputedPropertyName(property.key)
      : propertyName(property.key);
    if (name) recordDirect(bindingIdentifier(property.value), source, name);
  }
}

function stableConstInitializer(node, context, seen) {
  if (node?.type !== "Identifier") return null;
  const variable = resolveVariable(node, context);
  if (
    !variable ||
    seen.has(variable) ||
    variable.defs.length !== 1 ||
    isReassigned(node, context)
  ) {
    return null;
  }
  const def = variable.defs[0];
  if (
    def.type !== "Variable" ||
    def.parent?.kind !== "const" ||
    def.node?.id?.type !== "Identifier"
  ) {
    return null;
  }
  seen.add(variable);
  return def.node.init;
}

function namespaceSourceFromInit(init, context, namespaceBindings) {
  const expression = unwrapExpression(init);
  if (expression?.type !== "Identifier") return null;
  return namespaceBindings.get(resolveVariable(expression, context)) || null;
}

module.exports = {
  isLocalRequire,
  isReassigned,
  memberPropertyName,
  namespaceSourceFromInit,
  recordObjectPatternBindings,
  resolveVariable,
  resolveRuntimeVariable,
  stableConstInitializer,
};
