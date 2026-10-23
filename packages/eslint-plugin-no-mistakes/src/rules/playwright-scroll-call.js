"use strict";

const { resolveVariable } = require("./test-no-shared-state-aliases");
const { resolveAlias } = require("./playwright-static-call-alias");
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
  if (!invoked) return false;
  let callee = invoked.callee;
  while (callee?.type === "ChainExpression") callee = callee.expression;
  if (callee?.type === "Identifier") {
    const variable = resolveVariable(callee, context);
    if (!variable || variable.defs.length === 0) return RAW_SCROLL_NAMES.has(callee.name);
    const alias = resolveAlias(callee, context, new Set());
    if (!alias || !globalScrollInit(alias.declarator, context)) return false;
    // The local binding can rename the property. Only scroll, scrollBy, and scrollTo count.
    return alias.kind !== "key" || RAW_SCROLL_NAMES.has(alias.key);
  }
  if (!RAW_SCROLL_NAMES.has(methodFromCallee(callee, context))) return false;
  const object =
    callee.object?.type === "ChainExpression" ? callee.object.expression : callee.object;
  return object?.type === "Identifier" && object.name === "window" && unbound(object, context);
}

module.exports = { isRawScrollCall };
