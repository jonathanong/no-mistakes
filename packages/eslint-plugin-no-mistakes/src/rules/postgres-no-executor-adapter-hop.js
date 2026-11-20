"use strict";

const { localFunction } = require("./postgres-no-executor-adapter-target");

const MAX_HOPS = 8;
const active = new Set();

// A helper is proven when `visibleSqlTarget` proves it. That function owns the shared
// set, so the helper the call site checks counts toward the same depth bound.
function provenHelper(call, index, context, options, facts) {
  const fn = localFunction(call.callee, context, facts);
  if (!fn) return false;
  const { visibleSqlTarget } = require("./postgres-no-executor-adapter-visible");
  return visibleSqlTarget(call, index, undefined, context, options, false, facts) === true;
}

function enterHelper(fn) {
  if (active.has(fn) || active.size >= MAX_HOPS) return false;
  active.add(fn);
  return true;
}

function leaveHelper(fn) {
  active.delete(fn);
}

// The executor parameter passed on, unchanged, as a plain argument of a proven helper.
function forwardedHop(reference, context, options, facts) {
  const node = reference.identifier;
  const call = node.parent;
  if (call?.type !== "CallExpression") return false;
  return provenHelper(call, call.arguments.indexOf(node), context, options, facts);
}

module.exports = { enterHelper, forwardedHop, leaveHelper, provenHelper };
