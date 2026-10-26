"use strict";

const { resolveVariable } = require("./async-target-bindings");

// Scope facts are sequential: consult real bindings to reject hoisted/TDZ shadows.
function scalarConstAlias(node, context) {
  if (node.parent.kind !== "const" || node.init.type !== "Identifier") return false;
  const variable = resolveVariable(node.init, context);
  if (
    !variable ||
    variable.defs.length !== 1 ||
    variable.references.some((reference) => reference.isWrite() && !reference.init)
  )
    return false;
  const def = variable.defs[0];
  // Parameter initializers can run before a later binding is initialized.
  // Follow known nullable parameters only from their function body.
  if (def.type === "Parameter") return def.node.body.range[0] < node.init.range[0];
  return (
    def.type === "Variable" && def.parent.kind === "const" && def.node.range[1] < node.init.range[0]
  );
}

module.exports = { scalarConstAlias };
