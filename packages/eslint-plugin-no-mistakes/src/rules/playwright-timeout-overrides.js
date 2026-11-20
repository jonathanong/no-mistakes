"use strict";
const { unwrapExpression, isFunction } = require("./async-ast");
const { resolveVariable, isReassigned } = require("./async-target-bindings");

const TIMEOUT_KEYS = new Set(["setTimeout", "slow"]);

function createOverrideBan(context, option, bindings, evaluate, slowCondition, report) {
  const enabled = option.banOverrides === true;
  function concreteObject(node, seen = new Set()) {
    node = unwrapExpression(node);
    if (!node || seen.has(node)) return null;
    seen.add(node);
    if (node.type === "ObjectExpression") return node;
    if (isFunction(node) || node.type !== "Identifier") return null;
    const variable = resolveVariable(node, context);
    const def = variable?.defs[0];
    if (def?.type !== "Variable" || def.parent.kind !== "const" || isReassigned(node, context))
      return null;
    return concreteObject(def.node.init, seen);
  }
  function timeoutProperty(node, name) {
    const object = concreteObject(node);
    if (!object) return;
    const entry = evaluate(object).properties?.get("timeout");
    if (entry) report(entry.origin, "override", name);
  }
  function isCallee(node) {
    let current = node;
    while (current.parent?.type === "ChainExpression") current = current.parent;
    return current.parent?.type === "CallExpression" && current.parent.callee === current;
  }
  function isTimeoutBinding(node) {
    if (node.type !== "Identifier") return false;
    const parent = node.parent;
    const property =
      parent?.type === "AssignmentPattern" && parent.left === node ? parent.parent : parent;
    return (
      property?.type === "Property" &&
      property.value === (parent?.type === "AssignmentPattern" ? parent : node) &&
      property.parent?.type === "ObjectPattern"
    );
  }
  function observe(node, api) {
    if (!enabled || !api) return;
    if (bindings.registration(api)) {
      for (const argument of node.arguments) timeoutProperty(argument, "test timeout option");
      return;
    }
    if (api.kind !== "method") return;
    if (api.key === "setTimeout") {
      report(node, "override", "runtime timeout");
      return;
    }
    if (api.key === "slow") {
      if (slowCondition(node) !== false) report(node, "override", "slow effective timeout");
      return;
    }
    if (api.key === "configure" && api.owner?.path?.includes("describe"))
      timeoutProperty(node.arguments[0], "suite timeout");
  }
  function finish() {
    if (!enabled) return;
    walk(context, context.sourceCode.ast, (node) => {
      if (node.type === "MemberExpression") {
        if (isCallee(node)) return;
      } else if (!isTimeoutBinding(node)) return;
      const api = bindings.resolve(node);
      if (api?.kind === "method" && TIMEOUT_KEYS.has(api.key))
        report(node, "override", `${api.key} reference`);
    });
  }
  return { observe, finish };
}

function walk(context, node, visit, seen = new Set()) {
  if (!node || typeof node.type !== "string" || seen.has(node)) return;
  seen.add(node);
  visit(node);
  for (const key of context.sourceCode.visitorKeys[node.type] || []) {
    const value = node[key];
    if (Array.isArray(value)) {
      for (const child of value) walk(context, child, visit, seen);
    } else walk(context, value, visit, seen);
  }
}

module.exports = { createOverrideBan };
