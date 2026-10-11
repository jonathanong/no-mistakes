"use strict";
const { resolveVariable } = require("./async-target-bindings");
const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { memberName } = require("./vitest-timeout-copy-paths");

function laterTopLevelDeletion(member, deletion, context) {
  const receiver = unwrapExpression(member.object);
  const target = unwrapExpression(deletion.argument);
  if (
    receiver.type !== "Identifier" ||
    target.type !== "MemberExpression" ||
    unwrapExpression(target.object).type !== "Identifier" ||
    memberName(target, context) === null
  )
    return false;
  const variable = resolveVariable(receiver, context);
  if (variable !== resolveVariable(unwrapExpression(target.object), context)) return false;
  const def = variable?.defs[0];
  // Receiver aliases do not establish the lifetime of an own descriptor.
  if (
    def?.type !== "Variable" ||
    def.parent.kind !== "const" ||
    def.node.id.type !== "Identifier" ||
    !["ObjectExpression", "ArrayExpression"].includes(unwrapExpression(def.node.init)?.type)
  )
    return false;
  const assignment = unwrapTransparentParent(member).parent;
  if (assignment.type !== "AssignmentExpression" || unwrapExpression(assignment.left) !== member)
    return false;
  const before = unwrapTransparentParent(assignment).parent;
  const after = unwrapTransparentParent(deletion).parent;
  // Source offsets cannot order a callback, loop, branch or nested expression.
  return (
    before.type === "ExpressionStatement" &&
    after.type === "ExpressionStatement" &&
    before.parent.type === "Program" &&
    before.parent === after.parent &&
    before.parent.body.indexOf(before) < before.parent.body.indexOf(after)
  );
}
module.exports = { laterTopLevelDeletion };
