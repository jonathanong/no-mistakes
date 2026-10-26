"use strict";
const { resolveVariable, stableConstInitializer } = require("./async-target-bindings");
const { unwrapExpression } = require("./async-ast");
const { literalString } = require("./module-mock-helpers");
const {
  memberName,
  contextParameter,
  hasNonInitializingWrite,
  isContextObject,
} = require("./test-no-skips-bindings");

function bindsSkip(pattern, variable, context) {
  if (hasNonInitializingWrite(variable)) return false;
  return pattern.properties.some((property) => {
    if (property.type !== "Property") return false;
    const key = property.computed
      ? literalString(property.key)
      : (property.key.name ?? property.key.value);
    if (key !== "skip") return false;
    const value =
      property.value.type === "AssignmentPattern" ? property.value.left : property.value;
    return value.type === "Identifier" && resolveVariable(value, context) === variable;
  });
}

function isContextSkipAlias(node, context, seen = new Set()) {
  const variable = resolveVariable(node, context);
  if (!variable || seen.has(variable) || variable.defs.length !== 1) return false;
  const def = variable.defs[0];
  if (def.type === "Parameter") {
    const parameter = contextParameter(def.node, context);
    return parameter?.type === "ObjectPattern" && bindsSkip(parameter, variable, context);
  }
  if (def.type !== "Variable" || def.parent?.kind !== "const") return false;
  if (def.node.id.type === "ObjectPattern")
    return bindsSkip(def.node.id, variable, context) && isContextObject(def.node.init, context);
  const init = unwrapExpression(stableConstInitializer(node, context, seen));
  if (init?.type === "MemberExpression")
    return memberName(init, context) === "skip" && isContextObject(init.object, context);
  return init?.type === "Identifier" && isContextSkipAlias(init, context, seen);
}

module.exports = { isContextSkipAlias };
