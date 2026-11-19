"use strict";

const { childNodes, resolveVariable, unwrapTs } = require("./postgres-query-text");
const {
  acceptsExecutor,
  isImportedHelper,
  localFunction,
} = require("./postgres-no-executor-adapter-target");
const { callbackTarget } = require("./postgres-no-executor-adapter-callback");
const { literalSql } = require("./postgres-no-executor-adapter-sql");
const { localHoleSafe } = require("./postgres-no-executor-adapter-sql-origin");
const { enterHelper, forwardedHop, leaveHelper } = require("./postgres-no-executor-adapter-hop");
const { localExecutorType } = require("./postgres-no-executor-adapter-type");
const { passedOn } = require("./postgres-no-executor-adapter-option-forward");
const { importedVisible } = require("./postgres-no-executor-adapter-imported-visible");
const { libraryVisible } = require("./postgres-no-executor-adapter-library");

function opaqueBody(body) {
  const stack = [body];
  while (stack.length) {
    const node = stack.pop();
    if (node.type === "Identifier" && node.name === "arguments") return true;
    const callee = node.type === "CallExpression" ? unwrapTs(node.callee) : null;
    if (callee?.type === "Identifier" && callee.name === "eval") return true;
    stack.push(...childNodes(node));
  }
  return false;
}

function parameterBinding(param, property) {
  const target = param?.type === "AssignmentPattern" ? param.left : param;
  if (!property) return target?.type === "Identifier" ? { node: target, object: false } : null;
  if (target?.type === "Identifier") return { node: target, object: true };
  if (target?.type !== "ObjectPattern") return null;
  const field = target.properties.find(
    (entry) =>
      entry.type === "Property" &&
      !entry.computed &&
      entry.key.name === property &&
      entry.value.type === "Identifier",
  );
  return field ? { node: field.value, object: false } : null;
}

function executorCall(reference, property) {
  let node = reference.identifier;
  while (node.parent?.type === "TSNonNullExpression") node = node.parent;
  let callee = node;
  if (property && node.parent?.type === "MemberExpression" && node.parent.object === node) {
    if (node.parent.computed || node.parent.property.name !== property) return null;
    callee = node.parent;
  }
  while (callee.parent?.type === "TSNonNullExpression") callee = callee.parent;
  const call = callee.parent;
  return call?.type === "CallExpression" && call.callee === callee ? call : null;
}

function otherPropertyRead(reference, property) {
  const member = reference.identifier.parent;
  if (
    member?.type !== "MemberExpression" ||
    member.object !== reference.identifier ||
    member.computed ||
    member.property.name === property
  )
    return false;
  const parent = member.parent;
  if (parent?.type === "AssignmentExpression" && parent.left === member) return false;
  if (
    parent?.type === "UpdateExpression" ||
    (parent?.type === "UnaryExpression" && parent.operator === "delete")
  )
    return false;
  return true;
}

// The same judgement that leaves an adapter closure unreported where it is an
// argument: its receiver accepts it and the receiver's SQL is visible.
function closureProven(fn, context, options, facts) {
  const call = fn.parent;
  const at = call.arguments.indexOf(fn);
  return (
    (acceptsExecutor(call, at, null, context, options, facts) ||
      callbackTarget(call, at, context, facts)) &&
    visibleSqlTarget(call, at, null, context, options, true, facts, true) === true
  );
}

function visibleSqlTarget(call, index, quoteNode, context, options, accepted, facts, callback) {
  if (call.type !== "CallExpression") return null;
  if (call.arguments.some((argument) => argument.type === "SpreadElement")) return null;
  if (quoteNode && (quoteNode.type !== "Property" || quoteNode.computed)) return null;
  const fn = localFunction(call.callee, context, facts);
  if (!fn)
    return accepted && isImportedHelper(call.callee, context)
      ? (importedVisible(call, index, quoteNode?.key.name, context, options, facts) ?? false)
      : null;
  if (fn.params.some((param) => param.type === "RestElement")) return false;
  const property = quoteNode?.key.name;
  const param = fn.params[index];
  const target = param?.type === "AssignmentPattern" ? param.left : param;
  if (
    !property &&
    target?.typeAnnotation &&
    !(callback && target.typeAnnotation.typeAnnotation.type === "TSFunctionType") &&
    !localExecutorType(target.typeAnnotation.typeAnnotation, context, options)
  )
    return false;
  const binding = parameterBinding(fn.params[index], property);
  if (!binding) return false;
  const variable = resolveVariable(binding.node, context);
  if (!variable?.references.length) return false;
  if (opaqueBody(fn.body)) return false;
  if (!enterHelper(fn)) return false;
  try {
    return variable.references.every((reference) => {
      if (reference.init) return true;
      if (reference.isWrite()) return false;
      if (binding.object && otherPropertyRead(reference, property)) return true;
      const hole = (value) => localHoleSafe(value, call, fn, context, variable, property);
      const terminal =
        accepted && binding.object
          ? libraryVisible(reference.identifier, context, options, hole, (fn) =>
              closureProven(fn, context, options, facts),
            )
          : null;
      if (terminal !== null) return terminal;
      if (accepted && binding.object && passedOn(reference)) {
        const inner = reference.identifier.parent;
        const at = inner.arguments.indexOf(reference.identifier);
        const next = [inner, at, quoteNode, context, options, accepted, facts];
        return visibleSqlTarget(...next) === true;
      }
      const invocation = executorCall(reference, property);
      if (!invocation) return !property && forwardedHop(reference, context, options, facts);
      return literalSql(invocation.arguments[0], context, options, (hole) =>
        localHoleSafe(hole, call, fn, context, variable, property),
      );
    });
  } finally {
    leaveHelper(fn);
  }
}

module.exports = { visibleSqlTarget };
