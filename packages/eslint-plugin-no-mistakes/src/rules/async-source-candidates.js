"use strict";

const { resolveVariable, stableConstInitializer } = require("./async-target-bindings");

// Mirror the shapes valueType can follow without resolving or parsing imports.
// In particular, an import or function declaration alone has no receiving type.
function possibleValueType(value, context, seen = new Set()) {
  if (!value || seen.size >= 32 || seen.has(value)) return false;
  const next = new Set(seen).add(value);
  if (value.type === "Identifier") {
    const variable = resolveVariable(value, context);
    if (variable?.defs.length !== 1) return false;
    if (variable.defs[0].name?.typeAnnotation?.typeAnnotation) return true;
    if (variable.references.some((ref) => ref.isWrite() && !ref.init)) return false;
    const init = stableConstInitializer(value, context, new Set());
    return possibleValueType(init, context, next);
  }
  if (value.type === "CallExpression") return !value.optional;
  return value.type === "MemberExpression" && possibleValueType(value.object, context, next);
}

module.exports = { possibleValueType };
