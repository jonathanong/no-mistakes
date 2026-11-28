"use strict";

const { resolveVariable } = require("./async-target-bindings");
const { typeAnnotation } = require("../react-node-types");

function laterBindingStatus(identifier, context, reactTypeStatus, expressionStatus, facts, seen) {
  const variable = resolveVariable(identifier, context);
  const defs = variable?.defs ?? [];
  if (!defs.length || seen.has(variable)) return seen.has(variable) ? "unresolved" : null;
  if (!defs.every((def) => def.node?.range?.[0] > identifier.range[0])) return null;
  seen.add(variable);
  try {
    let unresolved = false;
    for (const def of defs) {
      if (def.type !== "Variable" || def.node?.id?.type !== "Identifier") continue;
      const type = typeAnnotation(def.node.id);
      const status = type
        ? reactTypeStatus(type, facts)
        : def.node.init
          ? expressionStatus(def.node.init, seen)
          : "no";
      if (status === "yes") return "yes";
      if (status === "unresolved") unresolved = true;
    }
    return unresolved ? "unresolved" : "no";
  } finally {
    seen.delete(variable);
  }
}

module.exports = { laterBindingStatus };
