"use strict";

const { unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");

// Scope facts are sequential: consult real bindings to reject hoisted/TDZ shadows.
function scalarConstAlias(node, context) {
  const init = unwrapExpression(node.init);
  if (node.parent.kind !== "const" || init?.type !== "Identifier") return false;
  const variable = resolveVariable(init, context);
  if (
    !variable ||
    variable.defs.length !== 1 ||
    variable.references.some((reference) => reference.isWrite() && !reference.init)
  )
    return false;
  const def = variable.defs[0];
  // Parameter initializers can run before a later binding is initialized.
  // Follow known nullable parameters only from their function body.
  if (def.type === "Parameter") return def.node.body.range[0] < init.range[0];
  return def.type === "Variable" && def.parent.kind === "const" && def.node.range[1] < init.range[0];
}

module.exports = { scalarConstAlias };
