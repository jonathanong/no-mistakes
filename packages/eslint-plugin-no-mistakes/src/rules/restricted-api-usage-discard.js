"use strict";

const TRANSPARENT = new Set([
  "AwaitExpression",
  "ChainExpression",
  "TSNonNullExpression",
  "TSAsExpression",
  "TSSatisfiesExpression",
  "TSTypeAssertion",
  "TSInstantiationExpression",
  "LogicalExpression",
]);

function resultDiscarded(call) {
  let current = call;
  for (;;) {
    const parent = current.parent;
    const type = parent.type;
    if (TRANSPARENT.has(type)) {
      current = parent;
    } else if (type === "SequenceExpression") {
      if (parent.expressions.at(-1) !== current) return true;
      current = parent;
    } else if (type === "ConditionalExpression" && parent.test !== current) {
      current = parent;
    } else {
      return (
        type === "ExpressionStatement" ||
        (type === "ForStatement" && (parent.init === current || parent.update === current)) ||
        (type === "UnaryExpression" && parent.operator === "void")
      );
    }
  }
}

module.exports = { resultDiscarded };
