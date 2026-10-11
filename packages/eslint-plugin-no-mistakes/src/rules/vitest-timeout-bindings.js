"use strict";
const { createBindingResolver } = require("./vitest-timeout-binding-resolver");
// One resolver per rule/context; scopes and in-progress cycles never cross requests.
const resolvers = new WeakMap();
function resolver(context) {
  if (!resolvers.has(context)) resolvers.set(context, createBindingResolver(context));
  return resolvers.get(context);
}
function configCall(node, context) {
  const api = resolver(context)(node.callee);
  return api?.kind === "config" ? api.name : null;
}
function overrideCall(node, context) {
  const api = resolver(context).invocation(node).api;
  return api?.kind === "setter"
    ? "runtime"
    : api?.kind === "hook"
      ? "hook"
      : api?.kind === "test"
        ? "test"
        : api?.kind === "unknown"
          ? "unknown"
          : null;
}
function staticPropertyName(node, context) {
  if (!node.computed) return node.key.name ?? String(node.key.value);
  if (node.key.type === "Literal") return String(node.key.value);
  return resolver(context).staticName(node.key);
}
function recordCall(node, context) {
  resolver(context).record(node);
}
function invocationArguments(node, context) {
  return resolver(context).invocation(node).args;
}
function callbackFunction(node, context) {
  return resolver(context).callback(node);
}
function opaqueCallback(node, context) {
  return resolver(context).opaqueCallback(node);
}
module.exports = {
  configCall,
  overrideCall,
  staticPropertyName,
  recordCall,
  invocationArguments,
  callbackFunction,
  opaqueCallback,
};
