"use strict";

const { resolveVariable, stableConstInitializer } = require("./async-target-bindings");

const sourceTypeShapes = new Set([
  "TSTypeReference",
  "TSTypeQuery",
  "TSTypeLiteral",
  "TSIntersectionType",
  "TSParenthesizedType",
]);

// Mirror the shapes valueType can follow without resolving or parsing imports.
// In particular, an import or function declaration alone has no receiving type.
function possibleValueType(value, context, seen = new Set()) {
  if (!value || seen.size >= 32 || seen.has(value)) return false;
  const next = new Set(seen).add(value);
  if (value.type === "Identifier") {
    const variable = resolveVariable(value, context);
    if (variable?.defs.length !== 1) return false;
    const type = variable.defs[0].name?.typeAnnotation?.typeAnnotation;
    // Arrays, function signatures, and primitive annotations cannot establish
    // targetType or a property contract in the bounded source type model.
    if (type) return sourceTypeShapes.has(type.type);
    if (variable.references.some((ref) => ref.isWrite() && !ref.init)) return false;
    const init = stableConstInitializer(value, context, new Set());
    return possibleValueType(init, context, next);
  }
  // declarations.callable only resolves identifier callees, including imported
  // factories with a return-type alias. Member calls and IIFEs cannot contribute.
  if (value.type === "CallExpression") return !value.optional && value.callee.type === "Identifier";
  return value.type === "MemberExpression" && possibleValueType(value.object, context, next);
}

module.exports = { possibleValueType };
