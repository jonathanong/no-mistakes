"use strict";

const { literalString } = require("./module-mock-helpers");
const { childNodes, resolveVariable, unwrapTs } = require("./postgres-query-text");

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

// A configured method named `call` is that method. Otherwise `.call` forwards
// and the SQL text is the second argument.
function invocation(callee, methods) {
  const node = unwrapTs(callee);
  if (!node || node.type !== "MemberExpression") {
    return null;
  }
  const name = staticMemberName(node);
  if (!name) {
    return null;
  }
  if (methods.has(name)) {
    const object = unwrapTs(node.object);
    if (object?.type !== "Identifier") {
      return null;
    }
    return { object, argument: 0 };
  }
  if (name !== "call") {
    return null;
  }
  const inner = unwrapTs(node.object);
  const method = staticMemberName(inner);
  if (!method) {
    return null;
  }
  if (!methods.has(method)) {
    return null;
  }
  const object = unwrapTs(inner.object);
  if (object?.type !== "Identifier") {
    return null;
  }
  return { object, argument: 1 };
}

function pieceMatches(call, invoked, tags, parameterName) {
  if (call.arguments.length !== invoked.argument + 1) return false;
  if (call.arguments.some((argument) => argument.type === "SpreadElement")) return false;
  const piece = unwrapTs(call.arguments[invoked.argument]);
  if (parameterName) return piece?.type === "Identifier" && piece.name === parameterName;
  return sqlPiece(piece, tags);
}

function directAppend(call, methods, tags, parameterName) {
  const invoked = invocation(call.callee, methods);
  if (!invoked || !pieceMatches(call, invoked, tags, parameterName)) return null;
  // `.call(receiver, text)` mutates the receiver, not the object the method was read from.
  if (invoked.argument === 1) {
    const receiver = unwrapTs(call.arguments[0]);
    if (receiver?.type !== "Identifier") {
      return null;
    }
    return receiver;
  }
  return invoked.object;
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
    return directAppend(inner, methods, tags, func.params[0].name);
  }
  return null;
}

function findAppend(node, methods, tags, context) {
  let builder = null;
  const visit = (current) => {
    if (isFunction(current)) {
      return;
    }
    if (!builder && current.type === "CallExpression") {
      const object =
        directAppend(current, methods, tags, null) ||
        forwarderAppend(current, methods, tags, context);
      if (object) {
        builder = { object, call: current };
      }
    }
    for (const child of childNodes(current)) visit(child);
  };
  visit(node);
  return builder;
}

module.exports = {
  findAppend,
};
