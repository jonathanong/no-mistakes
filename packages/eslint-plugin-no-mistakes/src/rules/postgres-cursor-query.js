"use strict";

const { isDiscardedSqlStatementAppend } = require("./postgres-cursor-append");
const { namedCursorImport, namespaceCursorMember } = require("./postgres-cursor-imports");

function transparentParent(node) {
  let current = node;
  while (
    current.parent &&
    (current.parent.type === "ChainExpression" ||
      current.parent.type === "TSAsExpression" ||
      current.parent.type === "TSSatisfiesExpression" ||
      current.parent.type === "TSTypeAssertion" ||
      current.parent.type === "TSNonNullExpression") &&
    current.parent.expression === current
  ) {
    current = current.parent;
  }
  return current;
}

function directCallParent(node) {
  const current = transparentParent(node);
  return current.parent?.type === "CallExpression" && current.parent.callee === current
    ? current.parent
    : null;
}

function isTypeQuery(node) {
  let current = transparentParent(node);
  while (current.parent?.type === "TSQualifiedName") current = current.parent;
  return current.parent?.type === "TSTypeQuery";
}

function firstQuasiText(template, allowRaw) {
  const quasi = template?.quasis?.[0];
  if (!quasi) return null;
  return quasi.value?.cooked ?? (allowRaw ? (quasi.value?.raw ?? null) : null);
}

function exactSqlTag(context, tag, helpers, config) {
  const identifier = helpers.unwrap(tag);
  if (identifier?.type !== "Identifier") return false;
  const variable = helpers.findVariable(context, identifier);
  const definition = variable?.defs.find((candidate) => {
    const specifier = candidate.node;
    const declaration = candidate.parent || specifier.parent;
    const source = declaration?.source?.value;
    return (
      candidate.type === "ImportBinding" &&
      specifier.type === "ImportDefaultSpecifier" &&
      specifier.importKind !== "type" &&
      declaration?.type === "ImportDeclaration" &&
      declaration.importKind !== "type" &&
      typeof source === "string" &&
      config.sqlTagModules.has(source)
    );
  });
  return Boolean(
    definition && variable && !variable.references.some((reference) => reference.isWrite()),
  );
}

function directQueryHead(context, node, helpers, config) {
  const value = helpers.unwrap(node);
  if (value?.type === "Literal") return typeof value.value === "string" ? value.value : null;
  if (value?.type === "TemplateLiteral") return firstQuasiText(value, true);
  if (
    value?.type === "TaggedTemplateExpression" &&
    exactSqlTag(context, value.tag, helpers, config)
  ) {
    return firstQuasiText(value.quasi, false);
  }
  return null;
}

function isCursorQueryArgument(context, identifier, helpers, config) {
  const argument = transparentParent(identifier);
  const call = argument.parent;
  if (call?.type !== "CallExpression" || call.arguments?.[0] !== argument) return false;
  const callee = helpers.unwrap(call.callee);
  return Boolean(
    namedCursorImport(context, callee, helpers, config) ||
    namespaceCursorMember(context, callee, helpers, config),
  );
}

function queryHead(context, node, helpers, config) {
  const direct = directQueryHead(context, node, helpers, config);
  if (direct !== null) return direct;
  const value = helpers.unwrap(node);
  if (value?.type !== "Identifier") return null;
  const variable = helpers.findVariable(context, value);
  const definitions = variable?.defs.filter((definition) => definition.type === "Variable") ?? [];
  if (definitions.length !== 1) return null;
  const declaration = definitions[0]?.node;
  if (
    declaration?.type !== "VariableDeclarator" ||
    declaration.parent?.type !== "VariableDeclaration" ||
    declaration.parent.kind !== "const" ||
    variable?.references.some((reference) => {
      const identifier = reference.identifier;
      return (
        identifier !== declaration.id &&
        !isTypeQuery(identifier) &&
        !isCursorQueryArgument(context, identifier, helpers, config) &&
        !isDiscardedSqlStatementAppend(identifier, helpers, transparentParent)
      );
    })
  ) {
    return null;
  }
  return directQueryHead(context, declaration.init, helpers, config);
}

module.exports = {
  directCallParent,
  firstQuasiText,
  isTypeQuery,
  queryHead,
  transparentParent,
};
