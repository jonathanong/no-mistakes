"use strict";
const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");

// Only direct module statements establish one execution order. Blocks and
// function bodies can repeat or run later even when their source ranges follow.
function laterTopLevelWrite(member, assignment, context) {
  const variable = resolveVariable(unwrapExpression(member.object), context);
  // An alias requires its own complete write proof; keep this exception local
  // to the binding that creates the copied object.
  if (unwrapExpression(variable.defs[0].node.init).type !== "ObjectExpression") return false;
  const operation = unwrapTransparentParent(member).parent;
  const current = operation.parent;
  const later = assignment.parent;
  return (
    current.type === "ExpressionStatement" &&
    later.type === "ExpressionStatement" &&
    current.parent.type === "Program" &&
    current.parent === later.parent &&
    current.parent.body.indexOf(current) < current.parent.body.indexOf(later)
  );
}
module.exports = { laterTopLevelWrite };
