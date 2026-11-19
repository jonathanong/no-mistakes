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

// `if (!row)` / `if (row == null)` only reads the binding. Whether the consequent
// exits is left to the checker: the projection type stays nullable unless it does.
function nullishGuardRead(identifier) {
  const test = guardTest(identifier);
  return test?.parent.type === "IfStatement";
}

module.exports = { nullishGuardRead };
