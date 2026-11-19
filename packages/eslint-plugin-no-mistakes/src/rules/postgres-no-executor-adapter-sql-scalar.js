"use strict";

const { typedScalarProjection } = require("./postgres-no-executor-adapter-sql-type");
const {
  bindValueType,
  freshDate,
  freshLocal,
} = require("./postgres-no-executor-adapter-sql-fresh");
const { derivedLocal, derivedValue } = require("./postgres-no-executor-adapter-sql-mapped");
const { loopBinding } = require("./postgres-no-executor-adapter-sql-loop");
const { primitiveType } = require("./postgres-no-executor-adapter-sql-primitive");
const { executorResult, localReceiver } = require("./postgres-no-executor-adapter-sql-local");
const { objectMutation } = require("./postgres-no-executor-adapter-object-mutation");
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

const ROW_METHODS = new Set(["map", "flatMap", "forEach", "filter", "find", "some", "every"]);

// `{ at }` on `rows.map` is a field of the element, not a free Date or primitive parameter.
function rowCallbackField(def) {
  if (def.type !== "Parameter" || def.name?.parent?.type !== "Property") return null;
  const property = def.name.parent;
  if (property.computed || property.value !== def.name || property.key.type !== "Identifier")
    return null;
  const pattern = property.parent;
  const fn = def.node;
  const param = fn?.params?.[0];
  const target = param?.type === "AssignmentPattern" ? param.left : param;
  const call = fn?.parent;
  const callee = call?.type === "CallExpression" ? unwrapTs(call.callee) : null;
  if (
    pattern?.type !== "ObjectPattern" ||
    target !== pattern ||
    !call?.arguments.includes(fn) ||
    callee?.type !== "MemberExpression" ||
    callee.computed ||
    callee.optional ||
    callee.property.type !== "Identifier" ||
    !ROW_METHODS.has(callee.property.name) ||
    callee.object.type !== "Identifier"
  )
    return null;
  return { field: property.key.name, receiver: callee.object };
}

function elementFieldScalar(receiver, field, context, allowUndefined) {
  const variable = resolveVariable(receiver, context);
  const rowBacked = Boolean(
    variable?.defs.length &&
    (variable.defs.every((def) => def.type === "Parameter") ||
      executorResult(variable, context) ||
      localReceiver(variable, context, receiver)),
  );
  if (!rowBacked) return null;
  if (objectMutation(variable, null, null, receiver, context, receiver)) return false;
  const services = context.sourceCode.parserServices;
  if (!services?.program || !services.esTreeNodeToTSNodeMap?.has(receiver)) return false;
  const ts = require("typescript");
  const checker = services.program.getTypeChecker();
  const type = checker.getTypeAtLocation(services.esTreeNodeToTSNodeMap.get(receiver));
  const element = type.getNumberIndexType?.();
  if (!element) return false;
  const parts = element.isUnion() ? element.types : [element];
  const source = services.esTreeNodeToTSNodeMap.get(receiver);
  return parts.every((part) => {
    const property = checker.getPropertyOfType(part, field);
    if (!property || property.flags & ts.SymbolFlags.Optional) return false;
    return primitiveType(checker.getTypeOfSymbolAtLocation(property, source), ts, allowUndefined);
  });
}

// Lazy: sql-projection requires this module. A typed receiver or a const object literal.
function projectedField(receiver, field, context, allowUndefined) {
  return (
    typedScalarProjection(receiver, field, context, allowUndefined) ||
    require("./postgres-no-executor-adapter-sql-projection").projectedScalar(
      receiver,
      field,
      context,
      allowUndefined,
    )
  );
}

function destructuredField(variable, def, context, allowUndefined) {
  const { localField } = require("./postgres-no-executor-adapter-sql-projection");
  const field = def.node.init ? localField(def.node.id, variable, context) : null;
  return field === null ? null : projectedField(def.node.init, field, context, allowUndefined);
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
    return projectedField(node.object, node.property.name, context, allowUndefined);
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
          const field = destructuredField(variable, def, context, allowUndefined);
          if (field) return true;
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
      const field = variable.defs.length === 1 ? rowCallbackField(variable.defs[0]) : null;
      if (field) {
        const proven = elementFieldScalar(field.receiver, field.field, context, allowUndefined);
        if (proven !== null) return proven;
      }
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
