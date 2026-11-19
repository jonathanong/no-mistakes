"use strict";

const EQUALITY = new Set(["==", "===", "!=", "!=="]);

function nullish(node) {
  return (
    (node.type === "Literal" && node.raw === "null") ||
    (node.type === "Identifier" && node.name === "undefined")
  );
}

function guardTest(identifier) {
  const parent = identifier.parent;
  if (parent.type === "UnaryExpression") return parent.operator === "!" ? parent : null;
  const comparison =
    parent.type === "BinaryExpression" &&
    EQUALITY.has(parent.operator) &&
    parent.left === identifier &&
    nullish(parent.right);
  return comparison ? parent : null;
}

// `if (!row)` / `if (row == null)`, alone or as an operand of `||` / `&&`, only
// reads the binding. Whether the consequent exits is left to the checker: the
// projection type stays nullable unless every path that reaches it narrowed it.
function nullishGuardRead(identifier) {
  let test = guardTest(identifier);
  while (test?.parent.type === "LogicalExpression" && test.parent.operator !== "??") {
    test = test.parent;
  }
  return test?.parent.type === "IfStatement";
}

module.exports = { nullishGuardRead };
