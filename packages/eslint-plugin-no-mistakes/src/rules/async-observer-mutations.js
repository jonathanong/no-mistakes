"use strict";

const { unwrapExpression } = require("./async-ast");
const { resolveVariable, memberPropertyName } = require("./async-target-bindings");

function observerMutated(node, context, suffix = []) {
  function paths(value, seen = new Set()) {
    value = unwrapExpression(value);
    if (value?.type === "MemberExpression") {
      return paths(value.object, seen).map((base) => [...base, memberPropertyName(value) ?? "*"]);
    }
    if (value?.type !== "Identifier") return [];
    const variable = resolveVariable(value, context);
    if (!variable?.defs.length) return [[value.name]];
    if (seen.has(variable)) return [[variable]];
    const next = new Set(seen).add(variable);
    const sources = variable.defs.map((def) => def.node.init).filter(Boolean);
    for (const reference of variable.references) {
      const assignment = reference.identifier.parent;
      if (
        reference.isWrite() &&
        assignment.type === "AssignmentExpression" &&
        assignment.left === reference.identifier
      )
        sources.push(assignment.right);
    }
    return [[variable], ...sources.flatMap((source) => paths(source, next))];
  }
  const roots = typeof node === "string" ? [[node]] : paths(node);
  if (!roots.length) return false;
  const observed = roots.map((root) => [...root, ...suffix]);
  function intersects(target, property = []) {
    return paths(target).some((root) => {
      const candidate = [...root, ...property];
      return observed.some(
        (expected) =>
          candidate[0] === expected[0] &&
          candidate
            .slice(1, Math.min(candidate.length, expected.length))
            .every(
              (part, index) =>
                part === "*" || expected[index + 1] === "*" || part === expected[index + 1],
            ),
      );
    });
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
    if (
      (value.type === "Property" &&
        value.parent.type === "ObjectExpression" &&
        intersects(value.value)) ||
      (value.type === "ArrayExpression" && value.elements.some((element) => intersects(element)))
    )
      written = true;
    if (value.type === "CallExpression") {
      const callee = unwrapExpression(value.callee);
      let reflected = false;
      for (const methodPath of paths(callee)) {
        const method = methodPath.at(-1);
        if (
          methodPath.length === 2 &&
          ["Object", "Reflect"].includes(methodPath[0]) &&
          [
            "assign",
            "defineProperty",
            "defineProperties",
            "set",
            "deleteProperty",
            "setPrototypeOf",
          ].includes(method)
        ) {
          reflected = true;
          if (
            intersects(
              value.arguments[0],
              ["defineProperty", "set", "deleteProperty"].includes(method)
                ? [value.arguments[1]?.value ?? "*"]
                : [],
            )
          )
            written = true;
        }
      }
      // An unknown callee can replace a handler through an escaped object.
      if (!reflected && value.arguments.some((argument) => intersects(argument))) written = true;
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
  return (
    observerMutated(name, context) ||
    ["globalThis", "global", "window", "self"].some((holder) =>
      observerMutated(holder, context, [name]),
    )
  );
}

module.exports = { observerMutated, globalObserverMutated };
