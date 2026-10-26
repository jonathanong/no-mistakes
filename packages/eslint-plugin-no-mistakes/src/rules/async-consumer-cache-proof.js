"use strict";

const { resolveVariable } = require("./async-target-bindings");

function mapCall(node, method, cache, key, context, count) {
  const member = node?.callee;
  const first = node?.arguments?.[0];
  return (
    node?.type === "CallExpression" &&
    !node.optional &&
    node.arguments.length === count &&
    member?.type === "MemberExpression" &&
    !member.computed &&
    !member.optional &&
    member.property.name === method &&
    member.object.type === "Identifier" &&
    resolveVariable(member.object, context) === cache &&
    first?.type === "Literal" &&
    typeof first.value === "string" &&
    first.value === key
  );
}

// This is deliberately a complete body grammar: any intervening effect, alias,
// escape, overwrite, early return, or exception may strand the stored promise.
function cachedBodyAwaited(fn, param, context) {
  if (!fn.async || fn.generator || fn.params.some((item) => item.type !== "Identifier"))
    return false;
  const body = fn.body?.type === "BlockStatement" ? fn.body.body : [];
  if (body.length !== 3) return false;
  const [declaration, guard, returned] = body;
  const entry = declaration.declarations?.[0];
  const init = entry?.init;
  if (
    declaration.type !== "VariableDeclaration" ||
    declaration.kind !== "const" ||
    declaration.declarations.length !== 1 ||
    entry.id.type !== "Identifier" ||
    init?.type !== "NewExpression" ||
    init.callee.type !== "Identifier" ||
    init.callee.name !== "Map" ||
    init.arguments.length !== 0 ||
    resolveVariable(init.callee, context)?.defs.length
  )
    return false;
  const cache = resolveVariable(entry.id, context);
  const test = guard.test;
  const inserted =
    guard.consequent?.type === "BlockStatement"
      ? guard.consequent.body.length === 1 && guard.consequent.body[0]
      : guard.consequent;
  const set = inserted?.expression;
  const key = set?.arguments?.[0]?.value;
  if (
    guard.type !== "IfStatement" ||
    guard.alternate ||
    test?.type !== "UnaryExpression" ||
    test.operator !== "!" ||
    inserted?.type !== "ExpressionStatement" ||
    !mapCall(test.argument, "has", cache, key, context, 1) ||
    !mapCall(set, "set", cache, key, context, 2) ||
    returned.type !== "ReturnStatement" ||
    returned.argument?.type !== "AwaitExpression" ||
    !mapCall(returned.argument.argument, "get", cache, key, context, 1)
  )
    return false;
  const invocation = set.arguments[1];
  const parameter = resolveVariable(param, context);
  const reads = parameter?.references.filter((ref) => !ref.init) || [];
  return (
    invocation?.type === "CallExpression" &&
    !invocation.optional &&
    invocation.arguments.length === 0 &&
    invocation.callee.type === "Identifier" &&
    resolveVariable(invocation.callee, context) === parameter &&
    reads.length === 1 &&
    !reads[0].isWrite() &&
    reads[0].identifier === invocation.callee
  );
}

module.exports = { cachedBodyAwaited };
