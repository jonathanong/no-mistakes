"use strict";

const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { resolveVariable, stableConstInitializer } = require("./async-target-bindings");
const { importedAwaited } = require("./async-consumer-imported-proof");
const { propertiesWritten } = require("./async-disposition-mocks");

function localFunction(id, context, seen = new Set()) {
  if (id?.type !== "Identifier") return null;
  const variable = resolveVariable(id, context);
  if (
    !variable ||
    seen.has(variable) ||
    variable.defs.length !== 1 ||
    variable.references.some((ref) => ref.isWrite() && !ref.init) ||
    propertiesWritten(id, context, new Set(), null, true)
  )
    return null;
  const def = variable.defs[0];
  if (def.type === "FunctionName" && def.node.type === "FunctionDeclaration") return def.node;
  const init = unwrapExpression(stableConstInitializer(id, context, seen));
  return ["ArrowFunctionExpression", "FunctionExpression"].includes(init?.type)
    ? init
    : localFunction(init, context, seen);
}

function localBodySafe(fn, context) {
  let safe = true;
  function visit(node) {
    if (!node?.type || !safe) return;
    if (
      node.type === "Identifier" &&
      (node.name === "arguments" ||
        (node.name === "eval" &&
          node.parent?.type === "CallExpression" &&
          node.parent.callee === node))
    )
      safe = false;
    for (const key of context.sourceCode.visitorKeys[node.type] || []) {
      const child = node[key];
      if (Array.isArray(child)) child.forEach(visit);
      else visit(child);
    }
  }
  visit(fn.body);
  return safe;
}

function locallyAwaited(call, index, context) {
  const fn = localFunction(unwrapExpression(call.callee), context);
  const param = fn?.params[index];
  if (
    !param ||
    param.type !== "Identifier" ||
    !localBodySafe(fn, context) ||
    fn.params.some((p) => p.type === "RestElement")
  )
    return false;
  const variable = resolveVariable(param, context);
  const reads = variable?.references.filter((ref) => !ref.init) || [];
  return (
    reads.length > 0 &&
    reads.every((ref) => {
      if (ref.isWrite()) return false;
      for (let owner = ref.identifier.parent; owner && owner !== fn; owner = owner.parent)
        if (
          ["ArrowFunctionExpression", "FunctionExpression", "FunctionDeclaration"].includes(
            owner.type,
          )
        )
          return false;
      const value = unwrapTransparentParent(ref.identifier);
      const invocation = value.parent;
      if (
        invocation?.type !== "CallExpression" ||
        invocation.callee !== value ||
        invocation.optional
      )
        return false;
      const result = unwrapTransparentParent(invocation);
      return result.parent?.type === "AwaitExpression" && result.parent.argument === result;
    })
  );
}

function createConsumerProof(context) {
  const results = new WeakMap();
  return function provenAwaitingConsumer(argument) {
    const value = unwrapTransparentParent(argument);
    const call = value.parent;
    if (
      call?.type !== "CallExpression" ||
      call.optional ||
      call.callee.type !== "Identifier" ||
      call.arguments.some((arg) => arg.type === "SpreadElement")
    )
      return false;
    const index = call.arguments.indexOf(value);
    const binding = resolveVariable(call.callee, context);
    if (index < 0 || !binding) return false;
    if (!results.has(binding)) results.set(binding, new Map());
    const slots = results.get(binding);
    if (!slots.has(index))
      slots.set(
        index,
        locallyAwaited(call, index, context) || importedAwaited(call, index, context),
      );
    return slots.get(index);
  };
}

module.exports = { createConsumerProof };
