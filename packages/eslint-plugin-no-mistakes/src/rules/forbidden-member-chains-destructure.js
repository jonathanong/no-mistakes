"use strict";

const { matches, expressionInfo } = require("./forbidden-member-chains-match");

const PARAM_PARENTS = new Set([
  "FunctionDeclaration",
  "FunctionExpression",
  "ArrowFunctionExpression",
]);

function patternKey(prop) {
  const key = prop.key;
  if (key.type === "Identifier") return key.name;
  if (key.type === "Literal" && (typeof key.value === "string" || typeof key.value === "number")) {
    return String(key.value);
  }
  return null;
}

function objectPattern(node) {
  let current = node;
  while (current?.type === "AssignmentPattern") current = current.left;
  return current?.type === "ObjectPattern" ? current : null;
}

function destructureVisitors({ compiled, context, suppressed, info }) {
  function report(node, chain) {
    if (suppressed(node)) return;
    context.report({ node, messageId: "forbiddenChain", data: { message: chain.message } });
  }

  // Computed keys are unproven, including string and numeric literals.
  // A chain already matched by an outer property is not reported again.
  function walk(pattern, chainInfo, blocked) {
    for (const prop of pattern.properties) {
      if (prop.type !== "Property" || prop.computed) continue;
      const name = patternKey(prop);
      if (name == null) continue;
      const next = { segments: chainInfo.segments.concat(name), rooted: chainInfo.rooted };
      const blockedHere = new Set(blocked);
      for (const chain of compiled) {
        if (blockedHere.has(chain) || !matches(next, chain)) continue;
        report(prop, chain);
        blockedHere.add(chain);
      }
      const nested = objectPattern(prop.value);
      if (nested) walk(nested, next, blockedHere);
    }
  }

  function check(pattern, receiver) {
    const chainInfo = expressionInfo(receiver, info);
    if (!chainInfo) return;
    walk(pattern, chainInfo, new Set());
  }

  return {
    VariableDeclarator(node) {
      const pattern = objectPattern(node.id);
      if (pattern && node.init) check(pattern, node.init);
    },
    AssignmentExpression(node) {
      const pattern = objectPattern(node.left);
      if (pattern) check(pattern, node.right);
    },
    AssignmentPattern(node) {
      if (!PARAM_PARENTS.has(node.parent?.type)) return;
      const pattern = objectPattern(node.left);
      if (pattern) check(pattern, node.right);
    },
  };
}

module.exports = { destructureVisitors };
