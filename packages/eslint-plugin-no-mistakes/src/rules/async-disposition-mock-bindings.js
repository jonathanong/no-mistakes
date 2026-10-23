"use strict";

const { findContainingFunction } = require("./async-ast");
const { resolveVariable, isReassigned } = require("./async-target-bindings");
const { laterInBlock, statement } = require("./async-disposition-flow");

// Accept one direct initialization of a previously uninitialized let. Requiring
// sibling statements proves both execute in the same ordered block.
function mockInitializer(node, context, seen) {
  const variable = resolveVariable(node, context);
  if (!variable || seen.has(variable) || variable.defs.length !== 1) return null;
  const def = variable.defs[0];
  if (
    def.type !== "Variable" ||
    !["const", "let"].includes(def.parent.kind) ||
    def.node.id.type !== "Identifier"
  )
    return null;
  if (!isReassigned(node, context)) return { init: def.node.init, assignment: null };
  if (def.parent.kind !== "let" || def.node.init) return null;
  const writes = variable.references.filter((reference) => reference.isWrite());
  if (writes.length !== 1) return null;
  const id = writes[0].identifier;
  const assignment = id.parent;
  if (
    assignment.type !== "AssignmentExpression" ||
    assignment.operator !== "=" ||
    assignment.left !== id ||
    assignment.parent.type !== "ExpressionStatement" ||
    findContainingFunction(def.node) !== findContainingFunction(assignment) ||
    def.node.range[0] >= assignment.range[0] ||
    !laterInBlock(assignment, node) ||
    statement(assignment) !== assignment.parent
  )
    return null;
  return { init: assignment.right, assignment };
}

module.exports = { mockInitializer };
