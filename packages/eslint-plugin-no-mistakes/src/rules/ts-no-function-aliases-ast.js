"use strict";

function unwrapExpression(expression) {
  let current = expression;
  while (
    current &&
    (current.type === "AwaitExpression" ||
      current.type === "ChainExpression" ||
      [
        "TSAsExpression",
        "TSNonNullExpression",
        "TSSatisfiesExpression",
        "TSTypeAssertion",
        "TSInstantiationExpression",
      ].includes(current.type))
  ) {
    current = current.type === "AwaitExpression" ? current.argument : current.expression;
  }
  return current;
}

function identifierName(node) {
  return node && node.type === "Identifier" ? node.name : null;
}

const BINDING_WRAPPERS = new Set([
  "ChainExpression",
  "ParenthesizedExpression",
  "TSAsExpression",
  "TSNonNullExpression",
  "TSSatisfiesExpression",
  "TSTypeAssertion",
  "TSInstantiationExpression",
]);

function bindingSite(node) {
  let current = node;
  while (BINDING_WRAPPERS.has(current?.parent?.type)) current = current.parent;
  return { parent: current?.parent ?? null, node: current };
}

function staticMemberName(member) {
  const property = unwrapExpression(member?.property);
  if (!member?.computed) return identifierName(property);
  if (property?.type === "Literal" && ["string", "number"].includes(typeof property.value)) {
    return String(property.value);
  }
  if (property?.type === "TemplateLiteral" && property.expressions.length === 0) {
    return property.quasis[0].value.cooked;
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
      const name = identifierName(param.argument);
      return (
        name !== null &&
        args[index].type === "SpreadElement" &&
        name === identifierName(args[index].argument)
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
  bindingSite,
};
