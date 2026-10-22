"use strict";

const { resolveVariable } = require("./test-no-shared-state-aliases");
const { invokedCall, memberName, methodFromCallee } = require("./playwright-static-call");

const RAW_SCROLL_NAMES = new Set(["scrollTo", "scrollBy", "scroll"]);

function unbound(node, context) {
  return !(resolveVariable(node, context)?.defs?.length > 0);
}

function globalScrollInit(declarator, context) {
  const init = declarator.init;
  if (init?.type === "Identifier" && RAW_SCROLL_NAMES.has(init.name)) return unbound(init, context);
  let object = init;
  while (object?.type === "ChainExpression") object = object.expression;
  if (object?.type === "MemberExpression") {
    const receiver =
      object.object?.type === "ChainExpression" ? object.object.expression : object.object;
    return (
      receiver?.type === "Identifier" &&
      receiver.name === "window" &&
      unbound(receiver, context) &&
      RAW_SCROLL_NAMES.has(memberName(object))
    );
  }
  return (
    declarator.id?.type === "ObjectPattern" &&
    init?.type === "Identifier" &&
    init.name === "window" &&
    unbound(init, context)
  );
}

function isRawScrollCall(node, context) {
  const invoked = invokedCall(node);
  if (!invoked || !RAW_SCROLL_NAMES.has(methodFromCallee(invoked.callee, context))) return false;
  let callee = invoked.callee;
  while (callee?.type === "ChainExpression") callee = callee.expression;
  if (callee?.type === "Identifier") {
    const variable = resolveVariable(callee, context);
    if (!variable || variable.defs.length === 0) return true;
    if (variable.defs.length !== 1) return false;
    const def = variable.defs[0];
    if (def.type !== "Variable" || def.parent?.kind !== "const") return false;
    return globalScrollInit(def.node, context);
  }
  const object =
    callee.object?.type === "ChainExpression" ? callee.object.expression : callee.object;
  return object?.type === "Identifier" && object.name === "window" && unbound(object, context);
}

module.exports = { isRawScrollCall };
