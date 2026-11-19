"use strict";

const { localFunction } = require("./postgres-no-executor-adapter-target");

const MAX_HOPS = 8;
const active = new Set();

// A helper is proven when `visibleSqlTarget` proves it. The shared set stops cycles
// and bounds depth, so every hop runs the same SQL literal, hole and executor-use checks.
function provenHelper(call, index, context, options, facts) {
  const fn = localFunction(call.callee, context, facts);
  if (!fn || active.has(fn) || active.size >= MAX_HOPS) return false;
  active.add(fn);
  try {
    const { visibleSqlTarget } = require("./postgres-no-executor-adapter-visible");
    return visibleSqlTarget(call, index, undefined, context, options, false, facts) === true;
  } finally {
    active.delete(fn);
  }
}

// The executor parameter passed on, unchanged, as a plain argument of a proven helper.
function forwardedHop(reference, context, options, facts) {
  const node = reference.identifier;
  const call = node.parent;
  if (call?.type !== "CallExpression" || call.callee === node) return false;
  return provenHelper(call, call.arguments.indexOf(node), context, options, facts);
}

module.exports = { forwardedHop, provenHelper };
