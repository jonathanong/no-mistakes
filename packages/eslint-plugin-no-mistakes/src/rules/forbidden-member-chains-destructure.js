"use strict";

const { disposition, expressionInfo, unwrap } = require("./forbidden-member-chains-match");

function patternKey(prop) {
  const key = prop.key;
  if (key.type === "Identifier") return key.name;
  if (key.type === "Literal" && (typeof key.value === "string" || typeof key.value === "number")) {
    return String(key.value);
  }
  return null;
}

// A resolved const string or number is that segment. An identifier the rule
// cannot read is a hole. Literal and other computed keys stay unproven.
function patternSegment(prop, resolveName) {
  if (!prop.computed) {
    const name = patternKey(prop);
    return name == null ? undefined : name;
  }
  const resolved = resolveName(prop.key);
  if (typeof resolved === "string") return resolved;
  if (unwrap(prop.key)?.type === "Identifier" && resolved == null) return null;
  return undefined;
}

function objectPattern(node) {
  let current = node;
  while (current?.type === "AssignmentPattern") current = current.left;
  return current?.type === "ObjectPattern" ? current : null;
}

function destructureVisitors({ compiled, context, suppressed, info, resolveName }) {
  // The outer pattern and a default can reach the same property and chain.
  // `const { panel: { open } = other.panel } = other` reports that pair once.
  const reported = new WeakMap();

  function report(node, chain, kind) {
    if (suppressed(node)) return;
    let seen = reported.get(node);
    if (seen?.has(chain)) return;
    if (!seen) {
      seen = new Set();
      reported.set(node, seen);
    }
    seen.add(chain);
    if (kind === "match") {
      context.report({ node, messageId: "forbiddenChain", data: { message: chain.message } });
      return;
    }
    context.report({
      node,
      messageId: "unanalyzable",
      data: { path: chain.path, message: chain.message },
    });
  }

  // A chain already matched by an outer property is not reported again.
  function walk(pattern, chainInfo, blocked) {
    for (const prop of pattern.properties) {
      if (prop.type !== "Property") continue;
      const name = patternSegment(prop, resolveName);
      if (name === undefined) continue;
      const next = { segments: chainInfo.segments.concat(name), rooted: chainInfo.rooted };
      const blockedHere = new Set(blocked);
      for (const chain of compiled) {
        const kind = disposition(next, chain);
        if (blockedHere.has(chain) || !kind) continue;
        report(prop, chain, kind);
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
