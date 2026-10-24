"use strict";

const { methodName, unwrap } = require("./no-object-mutation-syntax");
const { ARRAY_METHODS } = require("./no-object-mutation-ast");

// Limit the exemption to an inline, synchronous accumulator builder. Every
// callback exit must preserve the same accumulator for the next iteration.
function reducerCallback(node) {
  const callee = unwrap(node.callee);
  if (callee?.type !== "MemberExpression" || node.arguments.length !== 2) return null;
  if (!["reduce", "reduceRight"].includes(methodName(callee))) return null;
  const seed = unwrap(node.arguments[1]);
  if (!["ObjectExpression", "ArrayExpression"].includes(seed?.type)) return null;
  const callback = unwrap(node.arguments[0]);
  if (!["ArrowFunctionExpression", "FunctionExpression"].includes(callback?.type)) return null;
  if (callback.async || callback.generator || callback.params[0]?.type !== "Identifier")
    return null;
  const name = callback.params[0].name;
  const body = callback.body;
  if (body.type !== "BlockStatement") return null;
  if (body.body.at(-1)?.type !== "ReturnStatement") return null;
  return preservesAccumulator(body, name) ? callback : null;
}

function preservesAccumulator(node, name, nested = false) {
  if (!node || typeof node.type !== "string") return true;
  if (
    ["FunctionDeclaration", "FunctionExpression", "ArrowFunctionExpression"].includes(node.type)
  ) {
    // Returns in nested closures do not define the reduction result. Still
    // reject rebinding there: a closure can change the next accumulator.
    nested = true;
  }
  if (node.type === "ReturnStatement" && !nested) {
    const value = unwrap(node.argument);
    return value?.type === "Identifier" && value.name === name;
  }
  // Rebinding could pass a shared value into the following iteration. Keep
  // destructuring and shadowing conservative rather than prove control flow.
  if (
    [
      "AssignmentExpression",
      "UpdateExpression",
      "VariableDeclarator",
      "ForInStatement",
      "ForOfStatement",
    ].includes(node.type)
  ) {
    const target = node.left || node.argument || node.id;
    if (target.type !== "MemberExpression" && mentions(target, name)) return false;
  }
  if (node.type === "Identifier" && node.name === name && !directAccumulatorUse(node)) return false;
  if (node.type === "CatchClause" && mentions(node.param, name)) return false;
  for (const [key, value] of Object.entries(node)) {
    if (["parent", "loc", "range", "tokens"].includes(key)) continue;
    if (Array.isArray(value)) {
      if (!value.every((part) => preservesAccumulator(part, name, nested))) return false;
    } else if (value && typeof value === "object" && !preservesAccumulator(value, name, nested))
      return false;
  }
  return true;
}

// Only direct container writes are exempt. Member reads and aliases can carry
// shared nested provenance across arbitrarily many iterations, so abstain.
function directAccumulatorUse(node) {
  const parent = node.parent;
  if (parent.type === "ReturnStatement") return true;
  if (parent.type === "CallExpression") return parent.arguments.includes(node);
  if (parent.type !== "MemberExpression") return false;
  if (parent.property === node && !parent.computed) return true;
  if (parent.object !== node) return false;
  const use = parent.parent;
  if (use.type === "AssignmentExpression" && use.left === parent) return true;
  if (use.type === "UpdateExpression" && use.argument === parent) return true;
  return (
    use.type === "CallExpression" && use.callee === parent && ARRAY_METHODS.has(methodName(parent))
  );
}

function mentions(node, name) {
  if (!node || typeof node.type !== "string") return false;
  if (node.type === "Identifier") return node.name === name;
  return Object.entries(node).some(([key, value]) => {
    if (["parent", "loc", "range", "tokens"].includes(key)) return false;
    return Array.isArray(value)
      ? value.some((part) => mentions(part, name))
      : mentions(value, name);
  });
}

module.exports = { reducerCallback };
