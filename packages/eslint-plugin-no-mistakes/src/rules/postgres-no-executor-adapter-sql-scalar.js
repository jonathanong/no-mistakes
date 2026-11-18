"use strict";

const { primitiveType, typedScalarProjection } = require("./postgres-no-executor-adapter-sql-type");
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

function scalarArgument(node, context, allowUndefined = true, seen = new Set()) {
  node = unwrapTs(node);
  if (node?.type === "MemberExpression" && !node.computed && !node.optional)
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
        if (def.type === "Variable") {
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
      if (variable.defs.some((def) => def.type !== "Parameter")) return false;
    }
  }
  const parameter = node?.type === "Identifier" ? resolveVariable(node, context) : null;
  if (!parameter?.defs.length || !parameter.defs.every((def) => def.type === "Parameter"))
    return false;
  const services = context.sourceCode.parserServices;
  if (!services?.program || !services.esTreeNodeToTSNodeMap?.has(node)) return false;
  const ts = require("typescript");
  const checker = services.program.getTypeChecker();
  return primitiveType(
    checker.getTypeAtLocation(services.esTreeNodeToTSNodeMap.get(node)),
    ts,
    allowUndefined,
  );
}

module.exports = { scalarArgument };
