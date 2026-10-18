"use strict";

const { matches, expressionInfo } = require("./forbidden-member-chains-match");

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
  // The outer pattern and a default can reach the same property and chain.
  // `const { panel: { open } = other.panel } = other` reports that pair once.
  const reported = new WeakMap();

  function report(node, chain) {
    if (suppressed(node)) return;
    let seen = reported.get(node);
    if (seen?.has(chain)) return;
    if (!seen) {
      seen = new Set();
      reported.set(node, seen);
    }
    seen.add(chain);
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
    // A default is a receiver at any depth, not only on a function parameter.
    // The parser only allows one in a parameter, property, or array element.
    // `const { x: { open } = other.panel } = {}` reports `*.panel.open`.
    // An array element is too: `const [{ open } = other.panel] = []`.
    AssignmentPattern(node) {
      const pattern = objectPattern(node.left);
      if (pattern) check(pattern, node.right);
    },
  };
}

module.exports = { destructureVisitors };
