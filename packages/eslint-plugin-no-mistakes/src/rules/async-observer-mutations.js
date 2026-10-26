"use strict";

const { unwrapExpression } = require("./async-ast");
const {
  resolveVariable,
  stableConstInitializer,
  memberPropertyName,
} = require("./async-target-bindings");

function observerMutated(node, context, suffix = []) {
  function path(value, seen = new Set()) {
    value = unwrapExpression(value);
    if (value?.type === "MemberExpression") {
      const base = path(value.object, seen);
      return base && [...base, memberPropertyName(value) ?? "*"];
    }
    if (value?.type !== "Identifier") return null;
    const variable = resolveVariable(value, context);
    const init = stableConstInitializer(value, context, seen);
    return (init && path(init, seen)) || [variable?.defs.length ? variable : value.name];
  }
  const root = typeof node === "string" ? [node] : path(node);
  if (!root) return false;
  const observed = [...root, ...suffix];
  function intersects(target, property = []) {
    const root = path(target);
    const candidate = root && [...root, ...property];
    return (
      candidate &&
      candidate[0] === observed[0] &&
      candidate
        .slice(1, Math.min(candidate.length, observed.length))
        .every((part, index) => part === "*" || part === observed[index + 1])
    );
  }
  let written = false;
  function visit(value) {
    if (!value?.type || written) return;
    if (
      (value.type === "AssignmentExpression" && intersects(value.left)) ||
      (value.type === "UpdateExpression" && intersects(value.argument)) ||
      (value.type === "UnaryExpression" &&
        value.operator === "delete" &&
        intersects(value.argument))
    )
      written = true;
    if (value.type === "CallExpression") {
      const callee = unwrapExpression(value.callee);
      if (
        callee?.type === "MemberExpression" &&
        [
          "assign",
          "defineProperty",
          "defineProperties",
          "set",
          "deleteProperty",
          "setPrototypeOf",
        ].includes(memberPropertyName(callee)) &&
        intersects(
          value.arguments[0],
          ["defineProperty", "set", "deleteProperty"].includes(memberPropertyName(callee))
            ? [value.arguments[1]?.value ?? "*"]
            : [],
        )
      )
        written = true;
    }
    for (const key of context.sourceCode.visitorKeys[value.type] || []) {
      const child = value[key];
      if (Array.isArray(child)) child.forEach(visit);
      else visit(child);
    }
  }
  visit(context.sourceCode.ast);
  return written;
}

function globalObserverMutated(name, context) {
  return observerMutated(name, context) || observerMutated("globalThis", context, [name]);
}

module.exports = { observerMutated, globalObserverMutated };
