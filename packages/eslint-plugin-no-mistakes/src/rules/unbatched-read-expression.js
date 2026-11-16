"use strict";
const { isFunction, unwrapExpression } = require("./async-ast");

function optionalChain(node) {
  if (node?.type === "MemberExpression") return node.optional || optionalChain(node.object);
  if (node?.type === "CallExpression") return node.optional || optionalChain(node.callee);
  return false;
}

function conditionalChild(node, key) {
  if (node.type === "LogicalExpression") return key === "right";
  if (node.type === "ConditionalExpression") return key !== "test";
  if (node.type === "AssignmentPattern") return key === "right";
  if (node.type === "AssignmentExpression" && ["&&=", "||=", "??="].includes(node.operator))
    return key === "right";
  if (node.type === "CallExpression" && key === "arguments") return optionalChain(node);
  if (node.type === "MemberExpression" && key === "property") return optionalChain(node);
  return false;
}

function findAwaitedRead(expression, context, match) {
  const candidates = [];
  function visit(node, conditional) {
    if (!node || isFunction(node)) return;
    if (node.type === "AwaitExpression") {
      const read = match(node.argument);
      if (read)
        candidates.push({
          read,
          conditional:
            conditional || read.call.optional || optionalChain(unwrapExpression(read.call.callee)),
        });
    }
    for (const key of context.sourceCode.visitorKeys[node.type] || []) {
      const value = node[key];
      const nestedConditional = conditional || conditionalChild(node, key);
      if (Array.isArray(value)) {
        for (const child of value) {
          if (child?.type) visit(child, nestedConditional);
        }
      } else if (value?.type) {
        visit(value, nestedConditional);
      }
    }
  }
  visit(expression, false);
  return candidates.length === 1 && !candidates[0].conditional ? candidates[0].read : null;
}

module.exports = { findAwaitedRead };
