"use strict";

function isScopedExecutor(scoped, name, node) {
  const position = node?.range?.[0];
  return Boolean(
    scoped?.some(
      (entry) =>
        !entry.property &&
        entry.name === name &&
        entry.range[0] <= position &&
        position < entry.range[1],
    ),
  );
}

function isScopedMember(scoped, name, property, variable) {
  if (!property || variable?.defs?.length !== 1 || variable.defs[0].type !== "Parameter")
    return false;
  const range = variable.scope?.block?.range;
  return Boolean(
    range &&
    scoped?.some(
      (entry) =>
        entry.name === name &&
        entry.property === property &&
        entry.range[0] === range[0] &&
        entry.range[1] === range[1],
    ),
  );
}

module.exports = { isScopedExecutor, isScopedMember };
