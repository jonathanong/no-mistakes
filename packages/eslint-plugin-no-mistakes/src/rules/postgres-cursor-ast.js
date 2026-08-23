"use strict";

const WRAPPERS = new Set([
  "ChainExpression",
  "TSAsExpression",
  "TSSatisfiesExpression",
  "TSTypeAssertion",
  "TSNonNullExpression",
]);

function unwrap(node) {
  let current = node;
  while (current && WRAPPERS.has(current.type)) {
    current = current.expression;
  }
  return current;
}

function findVariable(context, identifier) {
  if (typeof identifier?.name !== "string") return null;
  let scope = context.sourceCode.getScope(identifier);
  while (scope) {
    const variable =
      (typeof scope.set?.get === "function" && scope.set.get(identifier.name)) ||
      scope.variables?.find((candidate) => candidate.name === identifier.name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function literalName(value) {
  if (
    typeof value === "string" ||
    typeof value === "number" ||
    typeof value === "boolean" ||
    typeof value === "bigint"
  ) {
    return value;
  }
  return null;
}

function staticPropertyName(node) {
  const value = unwrap(node);
  if (value?.type === "Literal") return literalName(value.value);
  if (value?.type === "TemplateLiteral" && (value.expressions ?? []).length === 0) {
    const quasi = (value.quasis ?? [])[0];
    return quasi?.value?.cooked ?? quasi?.value?.raw ?? null;
  }
  return null;
}

function propertyName(member) {
  if (member?.type !== "MemberExpression") return null;
  const property = member.property;
  if (!member.computed && property?.type === "Identifier" && typeof property.name === "string") {
    return property.name;
  }
  return member.computed ? staticPropertyName(property) : null;
}

function normalizeFilename(context) {
  const filename = String(context.filename ?? "").replaceAll("\\", "/");
  const cwd = context.cwd?.replaceAll("\\", "/").replace(/\/$/, "");
  return cwd && filename.startsWith(`${cwd}/`) ? filename.slice(cwd.length + 1) : filename;
}

module.exports = {
  findVariable,
  normalizeFilename,
  propertyName,
  unwrap,
};
