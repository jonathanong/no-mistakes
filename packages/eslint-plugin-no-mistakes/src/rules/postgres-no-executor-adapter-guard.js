"use strict";

const EQUALITY = new Set(["==", "===", "!=", "!=="]);

function nullish(node) {
  return (
    (node.type === "Literal" && node.value === null && node.raw === "null") ||
    (node.type === "Identifier" && node.name === "undefined")
  );
}

// `if (!row)` / `if (row == null)` only reads the binding. Whether the consequent
// exits is left to the checker: the projection type stays nullable unless it does.
function nullishGuardRead(identifier) {
  const parent = identifier.parent;
  let test = null;
  if (parent.type === "UnaryExpression" && parent.operator === "!") test = parent;
  else if (
    parent.type === "BinaryExpression" &&
    EQUALITY.has(parent.operator) &&
    nullish(parent.left === identifier ? parent.right : parent.left)
  )
    test = parent;
  return test?.parent.type === "IfStatement" && test.parent.test === test;
}

module.exports = { nullishGuardRead };
