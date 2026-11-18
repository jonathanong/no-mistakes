"use strict";

const api = require("./no-object-mutation-runtime");
const { escapeFresh, lookup, remember, unknown } = require("./no-object-mutation-values");
const {
  literalKey,
  memberKey,
  rootIdentifier,
  staticKey,
  staticPath,
  unwrap,
} = require("./no-object-mutation-syntax");

function markEscaped(state, tag, seen, direct = false) {
  if (tag?.intrinsic) state.builtins[tag.intrinsic].iterationChanged = true;
  const was = Boolean(tag?.type === "fresh" && tag.escaped);
  escapeFresh(tag, seen, state.returnEscapes, direct);
  if (!was && tag?.type === "fresh" && tag.escaped) {
    state.escapeEpoch += 1;
    api.escapeNestedFunctions(state, tag);
  }
}

function isAllowed(state, objectNode) {
  if (!state.options.allow.length || !objectNode) {
    return false;
  }
  const path = staticPath(objectNode);
  const root = rootIdentifier(objectNode);
  return state.options.allow.some((entry) => {
    if (root && entry === root) {
      return true;
    }
    return Boolean(path && (path === entry || path.startsWith(`${entry}.`)));
  });
}

function report(state, node, objectNode, viaCall) {
  if (state.reported.has(node)) {
    return;
  }
  if (viaCall && state.options.calls === false) {
    return;
  }
  if (isAllowed(state, objectNode)) {
    return;
  }
  if (state.context.suppressed(node)) {
    return;
  }
  state.reported.add(node);
  state.violations.push(node);
}

function shouldReport(state, target) {
  if (target.type === "mapset" || target.type === "fn" || target.type === "unknown") {
    return false;
  }
  if (target.type === "this") {
    // Class `this` is the instance under construction. A plain function's
    // `this` is whoever called it, which may be a shared object.
    if (!target.inClass) {
      return true;
    }
    return state.options.allowThis === false;
  }
  if (target.type === "fresh") {
    return true;
  }
  const enabled =
    target.origin === "module"
      ? state.options.module
      : target.origin === "import"
        ? state.options.imported
        : state.options.parameters;
  return enabled !== false;
}

function initializesOwnField(node) {
  const member = node.left;
  if (
    node.type !== "AssignmentExpression" ||
    node.operator !== "=" ||
    member?.type !== "MemberExpression" ||
    unwrap(member.object)?.type !== "ThisExpression"
  ) {
    return false;
  }
  let enclosing = node.parent;
  while (
    enclosing &&
    ![
      "FunctionExpression",
      "FunctionDeclaration",
      "ArrowFunctionExpression",
      "ClassDeclaration",
      "ClassExpression",
    ].includes(enclosing.type)
  ) {
    enclosing = enclosing.parent;
  }
  const method = enclosing?.parent;
  if (method?.type !== "MethodDefinition" || method.kind !== "constructor" || method.static) {
    return false;
  }
  const privateName = member.property.type === "PrivateIdentifier" ? member.property.name : null;
  const key = memberKey(member);
  return method.parent.body.some((field) => {
    if (field.type !== "PropertyDefinition" || field.static) return false;
    if (privateName !== null) {
      return field.key.type === "PrivateIdentifier" && field.key.name === privateName;
    }
    return key !== null && (field.computed ? literalKey(field.key) : staticKey(field.key)) === key;
  });
}

function considerWrite(state, target, node, info) {
  if (target?.intrinsic) state.builtins[target.intrinsic].iterationChanged = true;
  const path = info.objectNode && staticPath(info.objectNode);
  if (
    path &&
    (path === "Array.prototype" || path.startsWith("Array.prototype.")) &&
    !lookup(info.scope, "Array")
  ) {
    state.arrayPrototypeChanged = true;
  }
  if (!info.viaCall && initializesOwnField(node)) return;
  if (target?.mutator) target = target.mutator.member;
  if (target && (!info.viaCall || info.changesMethods)) target.iterationChanged = true;
  if (target?.type === "union") {
    for (const member of target.members) {
      considerWrite(state, member, node, info);
    }
    return;
  }
  if (target?.type === "fresh" && target.ownerEnv !== info.scope.ownerEnv && info.scope.fnTag) {
    info.scope.fnTag.captured.push(target);
    info.scope.fnTag.deferred.push({
      node,
      fresh: target,
      objectNode: info.objectNode,
      viaCall: Boolean(info.viaCall),
    });
    return;
  }
  if (target?.type === "fresh" && !target.escaped) {
    if (info.key) {
      target.fields.set(info.key, info.rhs === undefined ? unknown() : info.rhs);
    } else if (info.rhs) {
      remember(target, info.rhs);
    }
    for (const remembered of info.remembered || []) {
      remember(target, remembered);
    }
    return;
  }
  if (!shouldReport(state, target)) {
    return;
  }
  report(state, node, info.objectNode, Boolean(info.viaCall));
}

module.exports = {
  considerWrite,
  markEscaped,
  report,
};
