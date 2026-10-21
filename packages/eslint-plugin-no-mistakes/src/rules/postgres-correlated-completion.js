"use strict";

const SIMPLE = new Set([
  "ExpressionStatement",
  "VariableDeclaration",
  "EmptyStatement",
  "DebuggerStatement",
  "FunctionDeclaration",
]);

function endsWith(node, type) {
  if (node.type === type) return true;
  if (node.type !== "BlockStatement" || node.body.length === 0) return false;
  return (
    node.body.slice(0, -1).every((statement) => SIMPLE.has(statement.type)) &&
    endsWith(node.body.at(-1), type)
  );
}

function throwingChain(nodes) {
  let root = nodes.values().next().value;
  while (root.parent.type === "IfStatement" && root.parent.alternate === root) root = root.parent;
  let current = root,
    count = 0;
  while (current?.type === "IfStatement") {
    if (!nodes.has(current)) return false;
    count++;
    current = current.alternate;
  }
  return count === nodes.size && current != null && endsWith(current, "ThrowStatement");
}

function completeGuards(group, func) {
  const nodes = new Set(group.map((guard) => guard.node));
  if (throwingChain(nodes)) return true;
  if (func.body.type !== "BlockStatement") return false;
  const statements = func.body.body;
  if (!endsWith(statements.at(-1), "ThrowStatement")) return false;
  // Only direct returning guards and straight-line setup prove this finite shape.
  return (
    statements
      .slice(0, -1)
      .every((statement) =>
        nodes.has(statement)
          ? !statement.alternate && endsWith(statement.consequent, "ReturnStatement")
          : SIMPLE.has(statement.type),
      ) && group.every((guard) => guard.node.parent === func.body)
  );
}

module.exports = { completeGuards };
