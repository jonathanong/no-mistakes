"use strict";

const { literalString } = require("./module-mock-helpers");
const { resolveVariable, unwrapTs } = require("./postgres-query-text");

function memberName(node) {
  if (node?.type !== "MemberExpression") return null;
  return node.computed ? literalString(unwrapTs(node.property)) : node.property.name;
}

function methodReference(expression, methods, context, seen = new Set()) {
  const node = unwrapTs(expression);
  if (node?.type === "SequenceExpression")
    return methodReference(node.expressions.at(-1), methods, context, seen);
  const name = memberName(node);
  if (name && methods.has(name)) return unwrapTs(node.object)?.type === "Identifier" ? node : null;
  if (node?.type !== "Identifier") return null;
  const variable = resolveVariable(node, context);
  if (!variable || seen.has(variable) || variable.defs.length !== 1) return null;
  if (variable.references.some((reference) => reference.isWrite() && !reference.init)) return null;
  seen.add(variable);
  const def = variable.defs[0];
  if (def.parent?.kind !== "const" || def.node.id.type !== "Identifier") return null;
  return methodReference(def.node.init, methods, context, seen);
}

function appendInvocation(call, methods, context) {
  const node = unwrapTs(call.callee);
  const name = memberName(node);
  // Configured receiver methods win over Function.prototype forwarding.
  if (!name) return null;
  if (methods.has(name)) {
    const object = unwrapTs(node.object);
    if (object?.type !== "Identifier" || call.arguments.length !== 1) return null;
    const piece = call.arguments[0];
    return piece.type === "SpreadElement" ? null : { object, piece };
  }
  if (name !== "call" && name !== "apply") return null;
  if (!methodReference(node.object, methods, context)) return null;
  if (call.arguments.length !== 2 || call.arguments.some((arg) => arg.type === "SpreadElement"))
    return null;
  // The explicit receiver owns the append even when the method came from another builder.
  const object = unwrapTs(call.arguments[0]);
  if (object?.type !== "Identifier") return null;
  let piece = call.arguments[1];
  if (name === "apply") {
    const array = unwrapTs(piece);
    if (array?.type !== "ArrayExpression" || array.elements.length !== 1) return null;
    piece = array.elements[0];
    if (!piece) return null;
  }
  return piece.type === "SpreadElement" ? null : { object, piece };
}

module.exports = { appendInvocation };
