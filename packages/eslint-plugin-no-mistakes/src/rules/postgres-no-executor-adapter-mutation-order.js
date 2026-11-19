"use strict";

const FUNCTIONS = new Set([
  "FunctionDeclaration",
  "FunctionExpression",
  "ArrowFunctionExpression",
  "PropertyDefinition",
]);
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

function strip(node) {
  let current = node;
  while (
    current &&
    (current.type === "MemberExpression" ||
      current.type === "ChainExpression" ||
      current.type.startsWith("TS"))
  )
    current = current.object ?? current.expression;
  return current;
}

function lookup(scope, name) {
  for (let current = scope; current; current = current.upper) {
    const found = current.set.get(name);
    if (found) return found;
  }
  return null;
}

function unwrapExpression(node) {
  let current = node;
  while (
    current &&
    (current.type === "ParenthesizedExpression" ||
      current.type === "ChainExpression" ||
      current.type === "TSAsExpression" ||
      current.type === "TSNonNullExpression" ||
      current.type === "TSSatisfiesExpression" ||
      current.type === "TSTypeAssertion")
  )
    current = current.expression;
  return current;
}

// An awaited call allocates a new object. An alias of an outer binding does not.
function awaitedCall(node) {
  const awaited = unwrapExpression(node);
  if (awaited?.type !== "AwaitExpression") return false;
  return unwrapExpression(awaited.argument)?.type === "CallExpression";
}

// A const declared inside `loop` whose value is rooted at another binding declared
// inside the same loop is a new object on every iteration only when that root's
// initializer is an awaited call.
function freshIn(variable, loop) {
  const def = variable.defs.length === 1 ? variable.defs[0] : null;
  const inside = (node) => loop.range[0] <= node.range[0] && node.range[1] <= loop.range[1];
  if (def?.type !== "Variable" || def.parent.kind !== "const" || !inside(def.node)) return false;
  const root = strip(def.node.init);
  if (root?.type !== "Identifier") return false;
  const rootVariable = lookup(variable.scope, root.name);
  if (!rootVariable?.identifiers[0] || !inside(rootVariable.identifiers[0])) return false;
  const rootDef = rootVariable.defs.length === 1 ? rootVariable.defs[0] : null;
  return Boolean(rootDef?.type === "Variable" && awaitedCall(rootDef.node.init));
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
      !freshIn(variable, loop),
  );
}

module.exports = { mayPrecede };
