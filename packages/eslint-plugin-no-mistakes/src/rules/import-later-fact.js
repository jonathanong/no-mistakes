"use strict";

const { literalString } = require("./logical-call");
const { factAtPath, namespaceMember, patternPath } = require("./import-pattern-path");

function assignmentValue(reference) {
  const parent = reference.identifier?.parent;
  return parent?.type === "AssignmentExpression" && parent.left === reference.identifier
    ? parent.right
    : null;
}

function contains(root, node) {
  for (let current = node; current; current = current.parent) if (current === root) return true;
  return false;
}

function patternAssignment(reference) {
  let current = reference.identifier;
  while (current?.parent && current.parent.type !== "Program") {
    const parent = current.parent;
    if (
      parent.type === "AssignmentExpression" &&
      parent.left !== reference.identifier &&
      contains(parent.left, reference.identifier)
    )
      return parent;
    current = parent;
  }
  return null;
}

function patternFact(assignment, reference, context, seen, memo, importFact) {
  const source = importFact(assignment.right, context, new Set(seen), memo);
  if (!source || source.typeOnly) return null;
  const path = patternPath(assignment.left, reference.identifier.name, literalString, [], context);
  if (path == null || source.unresolved) return { unresolved: true };
  return factAtPath(source, path, namespaceMember) || { unresolved: true };
}

function writeFact(reference, context, seen, memo, importFact) {
  const direct = assignmentValue(reference);
  if (direct) return importFact(direct, context, new Set(seen), memo);
  const assignment = patternAssignment(reference);
  return assignment ? patternFact(assignment, reference, context, seen, memo, importFact) : null;
}

function hasImportWrite(variable, context, seen, memo, importFact) {
  return (variable.references ?? []).some((reference) => {
    if (!reference.isWrite() || reference.init) return false;
    return Boolean(writeFact(reference, context, seen, memo, importFact));
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
    const fact = writeFact(reference, context, seen, memo, importFact);
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
