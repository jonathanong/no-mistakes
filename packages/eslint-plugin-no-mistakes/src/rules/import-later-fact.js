"use strict";

function assignmentValue(reference) {
  const parent = reference.identifier?.parent;
  return parent?.type === "AssignmentExpression" && parent.left === reference.identifier
    ? parent.right
    : null;
}

function hasImportWrite(variable, context, seen, memo, importFact) {
  return (variable.references ?? []).some((reference) => {
    if (!reference.isWrite() || reference.init) return false;
    const value = assignmentValue(reference);
    return Boolean(value && importFact(value, context, new Set(seen), memo));
  });
}

function laterFact(def, variable, context, seen, memo, importFact, use) {
  const idType = def?.node?.id?.type;
  if (
    def?.type !== "Variable" ||
    (idType !== "Identifier" && idType !== "ObjectPattern" && idType !== "ArrayPattern")
  )
    return null;
  let matched = null;
  let noise = 0;
  for (const reference of variable.references ?? []) {
    if (!reference.isWrite() || reference.init) continue;
    if (use?.range && reference.identifier?.range?.[0] >= use.range[0]) continue;
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

module.exports = { hasImportWrite, laterFact };
