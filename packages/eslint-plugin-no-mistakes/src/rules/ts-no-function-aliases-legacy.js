"use strict";

// Keep the released unwrapping boundary here. Expanded analysis may accept
// additional syntax, but that must not change v0.95.0 identifier decisions.
function unwrapReleased(expression) {
  let current = expression;
  while (
    current &&
    (current.type === "AwaitExpression" ||
      current.type === "ChainExpression" ||
      current.type === "TSNonNullExpression")
  ) {
    current = current.type === "AwaitExpression" ? current.argument : current.expression;
  }
  return current;
}

function releasedCall(body) {
  if (!body) return null;
  if (body.type !== "BlockStatement") return unwrapReleased(body);
  if (body.body.length !== 1) return null;
  const statement = body.body[0];
  if (statement.type === "ReturnStatement") return unwrapReleased(statement.argument);
  if (statement.type === "ExpressionStatement") return unwrapReleased(statement.expression);
  return null;
}

function identifierName(node) {
  return node?.type === "Identifier" ? node.name : null;
}

function assignmentWrapperName(node) {
  if (node.parent?.type !== "AssignmentExpression" || node.parent.right !== node) return null;
  const left = node.parent.left;
  if (left.type === "Identifier") return left.name;
  if (left.type === "MemberExpression" && !left.computed) return identifierName(left.property);
  if (left.type === "MemberExpression" && left.property.type === "Literal") {
    return String(left.property.value);
  }
  return null;
}

function variableWrapperName(node) {
  return node.parent?.type === "VariableDeclarator" && node.parent.id.type === "Identifier"
    ? node.parent.id.name
    : null;
}

function sameArguments(params, args) {
  if (params.length !== args.length) return false;
  return params.every((param, index) => {
    if (param.type === "RestElement") {
      return (
        args[index].type === "SpreadElement" &&
        identifierName(param.argument) === identifierName(args[index].argument)
      );
    }
    if (param.type === "AssignmentPattern") return false;
    const name = identifierName(param);
    return name && args[index].type === "Identifier" && args[index].name === name;
  });
}

// null means the released rule did not own this callee shape. A bare identifier
// stays in the released lane even when that rule intentionally stayed silent.
function legacyIdentifierCall(node) {
  const call = releasedCall(node.body);
  if (call?.type !== "CallExpression") return null;
  const callee = identifierName(unwrapReleased(call.callee));
  if (!callee) return null;
  const wrapper = variableWrapperName(node);
  if (node.type !== "FunctionDeclaration" && !wrapper && !assignmentWrapperName(node)) {
    return false;
  }
  if ((wrapper || node.id?.name) === callee) return false;
  return sameArguments(node.params || [], call.arguments);
}

module.exports = { legacyIdentifierCall };
