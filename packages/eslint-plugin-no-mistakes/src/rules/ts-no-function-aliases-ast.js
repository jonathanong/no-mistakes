"use strict";

function unwrapExpression(expression) {
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

function identifierName(node) {
  return node && node.type === "Identifier" ? node.name : null;
}

function staticMemberName(member) {
  if (!member.computed) return identifierName(member.property);
  if (
    member.property.type === "Literal" &&
    ["string", "number"].includes(typeof member.property.value)
  ) {
    return String(member.property.value);
  }
  if (member.property.type === "TemplateLiteral" && member.property.expressions.length === 0) {
    const quasi = member.property.quasis[0];
    return quasi.value.cooked;
  }
  return null;
}

function callableTarget(expression) {
  const target = unwrapExpression(expression);
  if (identifierName(target)) return target;
  if (target?.type === "MemberExpression" && staticMemberName(target) !== null) return target;
  return null;
}

function isSameArgumentList(params, args) {
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

module.exports = {
  unwrapExpression,
  identifierName,
  staticMemberName,
  callableTarget,
  isSameArgumentList,
};
