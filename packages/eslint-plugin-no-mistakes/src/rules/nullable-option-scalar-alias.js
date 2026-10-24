"use strict";

const { isReassigned, resolveVariable } = require("./async-target-bindings");

// Scope facts are sequential: consult real bindings to reject hoisted/TDZ shadows.
function scalarConstAlias(node, context) {
  if (node.parent.kind !== "const" || node.init.type !== "Identifier") return false;
  const variable = resolveVariable(node.init, context);
  if (!variable || variable.defs.length !== 1 || isReassigned(node.init, context)) return false;
  const def = variable.defs[0];
  return (
    def.type === "Variable" && def.parent.kind === "const" && def.node.range[1] < node.init.range[0]
  );
}

module.exports = { scalarConstAlias };
