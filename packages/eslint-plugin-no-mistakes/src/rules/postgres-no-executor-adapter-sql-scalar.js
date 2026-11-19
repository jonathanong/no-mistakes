"use strict";

const { typedScalarProjection } = require("./postgres-no-executor-adapter-sql-type");
const {
  bindValueType,
  freshDate,
  freshLocal,
} = require("./postgres-no-executor-adapter-sql-fresh");
const { derivedLocal, derivedValue } = require("./postgres-no-executor-adapter-sql-mapped");
const { loopBinding } = require("./postgres-no-executor-adapter-sql-loop");
const { resolveVariable, unwrapTs } = require("./postgres-query-text");

function primitiveAnnotation(node, allowUndefined) {
  if (node?.type === "TSUnionType") {
    return (
      node.types.length > 0 && node.types.every((part) => primitiveAnnotation(part, allowUndefined))
    );
  }
  if (node?.type === "TSUndefinedKeyword") return allowUndefined;
  if (node?.type === "TSLiteralType") {
    return (
      node.literal?.type === "Literal" &&
      (node.literal.value === null ||
        ["string", "number", "boolean", "bigint"].includes(typeof node.literal.value))
    );
  }
  return [
    "TSStringKeyword",
    "TSNumberKeyword",
    "TSBooleanKeyword",
    "TSBigIntKeyword",
    "TSNullKeyword",
  ].includes(node?.type);
}

function typedBinding(def) {
  return def.type === "Parameter" || loopBinding(def);
}

function scalarArgument(node, context, allowUndefined = true, seen = new Set()) {
  node = unwrapTs(node);
  const scalar = (part) => scalarArgument(part, context, allowUndefined, seen);
  if (freshDate(node, context, scalar) || derivedValue(node, context, scalar)) return true;
  if (
    node?.type === "MemberExpression" &&
    !node.computed &&
    !node.optional &&
    node.property.type === "Identifier"
  )
    return typedScalarProjection(node.object, node.property.name, context, allowUndefined);
  if (node?.type === "Literal")
    return (
      node.value === null || ["string", "number", "boolean", "bigint"].includes(typeof node.value)
    );
  if (node?.type === "TemplateLiteral") return node.expressions.length === 0;
  if (node?.type === "TaggedTemplateExpression") return false;
  if (node?.type === "Identifier") {
    const variable = resolveVariable(node, context);
    if (variable && !seen.has(variable)) {
      seen.add(variable);
      if (variable.references.some((reference) => reference.isWrite() && !reference.init))
        return false;
      for (const def of variable.defs) {
        if (def.type === "Variable" && !loopBinding(def)) {
          const fresh = freshLocal(variable, def.node, node, context, scalar);
          if (fresh !== null) return fresh;
          const derived = derivedLocal(variable, def.node, node, context, scalar);
          if (derived !== null) return derived;
          return (
            Boolean(def.node.init) && scalarArgument(def.node.init, context, allowUndefined, seen)
          );
        }
        if (
          def.type === "Parameter" &&
          primitiveAnnotation(def.name?.typeAnnotation?.typeAnnotation, allowUndefined)
        )
          return true;
      }
      if (variable.defs.some((def) => def.type !== "Parameter" && !loopBinding(def))) return false;
    }
  }
  const parameter = node?.type === "Identifier" ? resolveVariable(node, context) : null;
  if (!parameter?.defs.length || !parameter.defs.every(typedBinding)) return false;
  const services = context.sourceCode.parserServices;
  if (!services?.program || !services.esTreeNodeToTSNodeMap?.has(node)) return false;
  const ts = require("typescript");
  const checker = services.program.getTypeChecker();
  return bindValueType(
    checker.getTypeAtLocation(services.esTreeNodeToTSNodeMap.get(node)),
    ts,
    services.program,
    allowUndefined,
  );
}

module.exports = { scalarArgument };
