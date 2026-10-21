"use strict";

const { unwrapTs } = require("./postgres-query-text");

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

function throwingChain(nodes, func) {
  let root = nodes.values().next().value;
  while (root.parent.type === "IfStatement" && root.parent.alternate === root) root = root.parent;
  let current = root,
    count = 0;
  while (current?.type === "IfStatement") {
    if (!nodes.has(current)) return false;
    count++;
    current = current.alternate;
  }
  return (
    count === nodes.size &&
    current != null &&
    endsWith(current, "ThrowStatement") &&
    reachedUnconditionally(root, func.body)
  );
}

function reachedUnconditionally(node, body) {
  while (node.parent.type === "BlockStatement") {
    const block = node.parent;
    const before = block.body.slice(0, block.body.indexOf(node));
    if (!before.every((statement) => SIMPLE.has(statement.type))) return false;
    if (block === body) return true;
    node = block;
  }
  return false;
}

function unconditionalAppend(node, call) {
  if (node.type === "ExpressionStatement") return unwrapTs(node.expression) === call;
  if (node.type === "ReturnStatement") return unwrapTs(node.argument) === call;
  if (node.type !== "BlockStatement") return false;
  for (const statement of node.body) {
    if (unconditionalAppend(statement, call)) return true;
    if (!SIMPLE.has(statement.type)) return false;
  }
  return false;
}

function completeGuards(group, func) {
  const nodes = new Set(group.map((guard) => guard.node));
  if (
    !group.every(
      (guard) =>
        !guard.call.optional &&
        !unwrapTs(guard.call.callee).optional &&
        unconditionalAppend(guard.node.consequent, guard.call),
    )
  )
    return false;
  if (throwingChain(nodes, func)) return true;
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
