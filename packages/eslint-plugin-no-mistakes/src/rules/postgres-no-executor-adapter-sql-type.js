"use strict";

const { resolveVariable } = require("./postgres-query-text");
const { objectMutation } = require("./postgres-no-executor-adapter-object-mutation");

function primitiveType(type, ts, allowUndefined) {
  if (type.isUnion())
    return (
      type.types.length > 0 && type.types.every((part) => primitiveType(part, ts, allowUndefined))
    );
  const mask =
    ts.TypeFlags.StringLike |
    ts.TypeFlags.NumberLike |
    ts.TypeFlags.BooleanLike |
    ts.TypeFlags.BigIntLike |
    ts.TypeFlags.Null |
    (allowUndefined ? ts.TypeFlags.Undefined : 0);
  return Boolean(type.flags & mask);
}

function typedScalarProjection(receiver, field, context, allowUndefined = true) {
  if (receiver.type !== "Identifier") return false;
  const variable = resolveVariable(receiver, context);
  if (
    !variable?.defs.length ||
    !variable.defs.every((def) => def.type === "Parameter") ||
    objectMutation(variable, null, null, receiver)
  )
    return false;
  const services = context.sourceCode.parserServices;
  if (!services?.program || !services.esTreeNodeToTSNodeMap?.has(receiver)) return false;
  const ts = require("typescript");
  const checker = services.program.getTypeChecker();
  const source = services.esTreeNodeToTSNodeMap.get(receiver);
  const type = checker.getTypeAtLocation(source);
  const parts = type.isUnion() ? type.types : [type];
  return parts.every((part) => {
    const property = checker.getPropertyOfType(part, field);
    if (!property || property.flags & ts.SymbolFlags.Optional) return false;
    if (property.declarations?.length !== 1) return false;
    const declaration = property.declarations[0];
    if (!(ts.isPropertySignature(declaration) || ts.isPropertyDeclaration(declaration)))
      return false;
    return primitiveType(checker.getTypeOfSymbolAtLocation(property, source), ts, allowUndefined);
  });
}

module.exports = { primitiveType, typedScalarProjection };
