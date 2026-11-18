"use strict";

const LOCAL_WRAPPERS = new Set([
  "TSAsExpression",
  "TSTypeAssertion",
  "TSNonNullExpression",
  "TSSatisfiesExpression",
  "TSInstantiationExpression",
  "ParenthesizedExpression",
  "ChainExpression",
]);

function objectMutation(variable, executorVariable, allowedProperty, ignoredReference) {
  return variable.references.some((reference) => {
    if (reference.init || reference.identifier === ignoredReference) return false;
    let value = reference.identifier;
    let depth = 0;
    while (true) {
      const parent = value.parent;
      if (parent?.expression === value && LOCAL_WRAPPERS.has(parent.type)) value = parent;
      else if (parent?.type === "MemberExpression" && parent.object === value) {
        value = parent;
        depth++;
      } else break;
    }
    const parent = value.parent;
    if (
      (parent?.type === "AssignmentExpression" && parent.left === value) ||
      (parent?.type === "UpdateExpression" && parent.argument === value) ||
      (parent?.type === "UnaryExpression" && parent.operator === "delete") ||
      (parent?.type === "VariableDeclarator" &&
        parent.init === value &&
        parent.id.type === "Identifier") ||
      (parent?.type === "CallExpression" &&
        (parent.arguments.includes(value) ||
          (parent.callee === value &&
            depth === 1 &&
            (variable !== executorVariable ||
              value.computed ||
              value.property.name !== allowedProperty))))
    )
      return true;
    if (depth > 0) return false;
    return !(
      parent?.type === "VariableDeclarator" &&
      parent.init === value &&
      parent.id.type === "ObjectPattern"
    );
  });
}

module.exports = { objectMutation };
