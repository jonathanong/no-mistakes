"use strict";

const TRANSPARENT = new Set([
  "AwaitExpression",
  "ChainExpression",
  "TSNonNullExpression",
  "TSAsExpression",
  "TSSatisfiesExpression",
  "TSTypeAssertion",
  "TSInstantiationExpression",
]);

function resultDiscarded(call) {
  let current = call;
  for (;;) {
    const parent = current.parent;
    if (!parent) return false;
    if (TRANSPARENT.has(parent.type)) {
      current = parent;
    } else if (parent.type === "SequenceExpression") {
      if (parent.expressions.at(-1) !== current) return true;
      current = parent;
    } else {
      return (
        parent.type === "ExpressionStatement" ||
        (parent.type === "UnaryExpression" && parent.operator === "void")
      );
    }
  }
}

module.exports = { resultDiscarded };
