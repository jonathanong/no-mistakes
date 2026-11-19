"use strict";

const { resolveVariable, unwrapTs } = require("./postgres-query-text");
const { objectMutation } = require("./postgres-no-executor-adapter-object-mutation");
const { executorResult, helperResult } = require("./postgres-no-executor-adapter-sql-local");

function singleExpressionCallback(call) {
  const [callback] = call.arguments;
  const simple =
    callback?.type === "ArrowFunctionExpression" &&
    call.arguments.length === 1 &&
    callback.params.length === 1 &&
    callback.params[0].type === "Identifier" &&
    !callback.async &&
    callback.body.type !== "BlockStatement";
  return simple ? callback.body : null;
}

function literalFields(body, scalar) {
  return body.properties.every(
    (entry) =>
      entry.type === "Property" && entry.kind === "init" && !entry.computed && scalar(entry.value),
  );
}

// `rows.map((r) => r.id)` or `rows.map((r) => ({ a: 1, b: r.id }))` over a never-written
// `const { rows } = await executor(...)`. The callback is one expression, so it holds no side
// effect, and each projected value passes the shared scalar proof.
function mappedRows(node, context, scalar) {
  const callee = node.type === "CallExpression" && !node.optional ? unwrapTs(node.callee) : null;
  if (
    callee?.type !== "MemberExpression" ||
    callee.computed ||
    callee.optional ||
    callee.property.name !== "map" ||
    callee.object.type !== "Identifier"
  )
    return false;
  const rows = resolveVariable(callee.object, context);
  const body = singleExpressionCallback(node);
  if (!body || !rows || !executorResult(rows, context)) return false;
  if (objectMutation(rows, null, null, callee.object, context, callee.object)) return false;
  return body.type === "ObjectExpression" ? literalFields(body, scalar) : scalar(body);
}

// An awaited proven helper result, or a mapped array of the executor's own rows.
function derivedValue(node, context, scalar) {
  return node.type === "AwaitExpression"
    ? helperResult(node, context)
    : mappedRows(node, context, scalar);
}

// A binding of such a value is proven unless a later use writes it.
function derivedLocal(variable, declarator, reference, context, scalar) {
  if (declarator.id.type !== "Identifier" || !declarator.init) return null;
  if (!derivedValue(unwrapTs(declarator.init), context, scalar)) return null;
  return !objectMutation(variable, null, null, reference, context);
}

module.exports = { derivedLocal, derivedValue };
