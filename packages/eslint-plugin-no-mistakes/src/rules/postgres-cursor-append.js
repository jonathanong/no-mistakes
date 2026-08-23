"use strict";

function appendCall(node, helpers, transparentParent) {
  const object = transparentParent(node);
  const member = object.parent;
  if (member?.type !== "MemberExpression" || member.object !== object) return null;
  if (member.computed || helpers.propertyName(member) !== "append") return null;
  const callee = transparentParent(member);
  const call = callee.parent;
  return call?.type === "CallExpression" && call.callee === callee ? call : null;
}

function isDiscardedSqlStatementAppend(identifier, helpers, transparentParent) {
  let call = appendCall(identifier, helpers, transparentParent);
  while (call) {
    const result = transparentParent(call);
    if (result.parent?.type === "ExpressionStatement") return true;
    call = appendCall(call, helpers, transparentParent);
  }
  return false;
}

module.exports = { isDiscardedSqlStatementAppend };
