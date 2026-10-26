"use strict";

const { literalString } = require("./module-mock-helpers");
const { childNodes, resolveVariable, unwrapTs } = require("./postgres-query-text");
const { appendInvocation } = require("./postgres-no-correlated-sql-builder-invocation");

function isFunction(node) {
  return (
    node?.type === "FunctionDeclaration" ||
    node?.type === "FunctionExpression" ||
    node?.type === "ArrowFunctionExpression"
  );
}

function staticMemberName(node) {
  if (node?.type !== "MemberExpression") {
    return null;
  }
  if (!node.computed) return node.property.name;
  return literalString(unwrapTs(node.property));
}

function sqlPiece(node, tags) {
  const value = unwrapTs(node);
  if (value.type === "Literal" && typeof value.value === "string") return true;
  if (value.type === "TemplateLiteral") return true;
  if (value.type !== "TaggedTemplateExpression") return false;
  const tag = unwrapTs(value.tag);
  if (tag?.type === "Identifier") return tags.has(tag.name);
  const name = staticMemberName(tag);
  return Boolean(name && tags.has(name));
}

function directAppend(call, methods, tags, parameterName, context) {
  const invoked = appendInvocation(call, methods, context);
  if (!invoked) return null;
  const piece = unwrapTs(invoked.piece);
  if (parameterName)
    return piece?.type === "Identifier" && piece.name === parameterName ? invoked.object : null;
  return sqlPiece(piece, tags) ? invoked.object : null;
}

function returnedCall(func) {
  if (func.body.type !== "BlockStatement") return unwrapTs(func.body);
  const statement = func.body.body.length === 1 ? func.body.body[0] : null;
  if (statement?.type !== "ReturnStatement") return null;
  return unwrapTs(statement.argument);
}

function functionFromBinding(identifier, context) {
  const variable = resolveVariable(identifier, context);
  if (!variable) {
    return null;
  }
  if (variable.defs.length !== 1) {
    return null;
  }
  const definition = variable.defs[0];
  if (definition.type === "FunctionName") {
    return isFunction(definition.node) ? definition.node : null;
  }
  if (definition.type !== "Variable" || definition.parent.kind !== "const") {
    return null;
  }
  const init = unwrapTs(definition.node.init);
  return isFunction(init) ? init : null;
}

function forwarderAppend(call, methods, tags, context) {
  const callee = unwrapTs(call.callee);
  if (callee?.type !== "Identifier") {
    return null;
  }
  if (call.arguments.length !== 1 || call.arguments[0].type === "SpreadElement") {
    return null;
  }
  if (!sqlPiece(call.arguments[0], tags)) {
    return null;
  }
  const func = functionFromBinding(callee, context);
  const inner =
    func && func.params.length === 1 && func.params[0].type === "Identifier"
      ? returnedCall(func)
      : null;
  if (inner?.type === "CallExpression") {
    return directAppend(inner, methods, tags, func.params[0].name, context);
  }
  return null;
}

function findAppends(node, methods, tags, context) {
  const builders = new Map();
  const visit = (current) => {
    if (isFunction(current)) {
      return;
    }
    if (current.type === "CallExpression") {
      const object =
        directAppend(current, methods, tags, null, context) ||
        forwarderAppend(current, methods, tags, context);
      const variable = object && resolveVariable(object, context);
      // One guard contributes once per actual receiver binding, in source order.
      if (variable && !builders.has(variable)) {
        builders.set(variable, { object, call: current });
      }
    }
    for (const child of childNodes(current)) visit(child);
  };
  visit(node);
  return [...builders.values()];
}

module.exports = {
  findAppends,
};
