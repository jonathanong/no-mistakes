"use strict";

const { resolveVariable } = require("./async-target-bindings");

function rememberStatus(statuses, identifier, status, context, keepPrior) {
  const variable = resolveVariable(identifier, context);
  if (!variable || (keepPrior && statuses.get(variable) === "yes")) return;
  statuses.set(variable, status);
}

function storedStatus(statuses, identifier, context) {
  return statuses.get(resolveVariable(identifier, context));
}

function reassignedBinding(identifier, context) {
  const variable = resolveVariable(identifier, context);
  if (!variable) return false;
  return variable.references.some((reference) => reference.isWrite() && !reference.init);
}

module.exports = { reassignedBinding, rememberStatus, storedStatus };
