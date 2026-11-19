"use strict";

const FUNCTIONS = new Set(["FunctionDeclaration", "FunctionExpression", "ArrowFunctionExpression"]);
const LOOPS = new Set([
  "WhileStatement",
  "DoWhileStatement",
  "ForStatement",
  "ForInStatement",
  "ForOfStatement",
]);

function enclosing(node, types) {
  const found = [];
  for (let current = node.parent; current; current = current.parent)
    if (types.has(current.type)) found.push(current);
  return found;
}

function enclosingFunction(node) {
  return enclosing(node, FUNCTIONS)[0] ?? null;
}

// Whether a use of a const local can still affect the value read at `before`.
// Closures and loops are conservative; see docs/eslint-rules/postgres-no-executor-adapter.md.
function mayPrecede(variable, reference, before) {
  if (!before) return true;
  const identifier = reference.identifier;
  const projectionFunction = enclosingFunction(before);
  const declarationBlock = variable.scope.variableScope.block;
  const declarationFunction = FUNCTIONS.has(declarationBlock.type) ? declarationBlock : null;
  if (projectionFunction !== declarationFunction) return true;
  if (enclosingFunction(identifier) !== projectionFunction) return true;
  if (identifier.range[0] < before.range[0]) return true;
  return enclosing(before, LOOPS).some(
    (loop) =>
      loop.range[0] <= identifier.range[0] &&
      identifier.range[1] <= loop.range[1] &&
      loop.range[0] >= variable.identifiers[0].range[1],
  );
}

module.exports = { mayPrecede };
