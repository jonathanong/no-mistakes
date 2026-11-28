"use strict";

function assignmentValue(reference) {
  const parent = reference.identifier?.parent;
  return parent?.type === "AssignmentExpression" && parent.left === reference.identifier
    ? parent.right
    : null;
}

function laterFact(def, variable, context, seen, memo, importFact) {
  if (def?.type !== "Variable" || def.node?.id?.type !== "Identifier") return null;
  let matched = null;
  let noise = 0;
  for (const reference of variable.references ?? []) {
    if (!reference.isWrite() || reference.init) continue;
    const value = assignmentValue(reference);
    const fact = value && importFact(value, context, new Set(seen), memo);
    if (!fact) {
      noise += 1;
      continue;
    }
    if (matched) noise += 1;
    else matched = fact;
  }
  if (!matched) return null;
  return noise || matched.unresolved ? { unresolved: true } : matched;
}

module.exports = { laterFact };
