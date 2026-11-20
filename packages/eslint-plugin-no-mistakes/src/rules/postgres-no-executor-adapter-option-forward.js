"use strict";

const { optionObjectType } = require("./postgres-no-executor-adapter-options");

const MAX_DEPTH = 4;
const chains = new WeakMap();

// Judge an options proof once per active chain: a revisited function (cycle)
// or a chain deeper than MAX_DEPTH stays unproven.
function guardedProof(fn, key, facts, prove) {
  if (!chains.has(facts)) chains.set(facts, new Set());
  const active = chains.get(facts);
  const id = `${fn.getSourceFile().fileName}:${fn.pos}:${key}`;
  if (active.has(id) || active.size >= MAX_DEPTH) return false;
  active.add(id);
  try {
    return prove();
  } finally {
    active.delete(id);
  }
}

// The unchanged binding is an argument of a direct call whose parameter has the
// same declared options type; the callee must then prove the property itself.
// Every other reference to the binding is judged by the caller's own proof, so
// a write, spread or store of the binding anywhere in the helper stays unproven.
function forwardedOption(identifier, property, checker, facts, tsOptionBody) {
  const ts = require("typescript");
  const call = identifier.parent;
  const index = call.arguments?.indexOf(identifier) ?? -1;
  if (index < 0 || ts.isNewExpression(call) || call.arguments.some(ts.isSpreadElement))
    return false;
  const callee = checker.getResolvedSignature(call)?.getDeclaration();
  if (!callee?.body || !ts.isFunctionDeclaration(callee) || callee.symbol.declarations.length !== 1)
    return false;
  const parameter = callee.parameters[index];
  const own = checker.getSymbolAtLocation(identifier)?.valueDeclaration;
  if (
    callee.parameters.some((entry) => entry.dotDotDotToken) ||
    typeof parameter?.name.escapedText !== "string" ||
    !parameter.type ||
    optionObjectType(parameter.type, checker) !== optionObjectType(own.type, checker)
  )
    return false;
  return tsOptionBody(callee, index, property, checker, facts);
}

// An accepted proof already judged every use of the binding, so a bare call
// argument here is a proven pass-on rather than an unseen executor call.
function passedOn(reference) {
  return reference.identifier.parent.type === "CallExpression";
}

function unwrapParens(node, ts) {
  let current = node;
  while (ts.isParenthesizedExpression(current.parent)) current = current.parent;
  return current;
}

// A write through another property of the binding changes what any callee it is
// passed on to sees, so it is never an unrelated read. Parentheses and
// destructuring-assignment targets (`(options.readOnly) = true`,
// `({ a: options.readOnly } = src)`, `({ a: (options.readOnly) } = src)`)
// are writes too. A shorthand pattern names an identifier, never this member.
function memberWrite(member) {
  const ts = require("typescript");
  const unwrapped = unwrapParens(member, ts);
  const parent = unwrapped.parent;
  if (
    (ts.isBinaryExpression(parent) &&
      parent.left === unwrapped &&
      ts.isAssignmentOperator(parent.operatorToken.kind)) ||
    ts.isPrefixUnaryExpression(parent) ||
    ts.isPostfixUnaryExpression(parent) ||
    ts.isDeleteExpression(parent)
  )
    return true;
  let cursor = member;
  for (;;) {
    const up = cursor.parent;
    if (!up) return false;
    if (ts.isParenthesizedExpression(up)) {
      cursor = up;
      continue;
    }
    if (ts.isPropertyAssignment(up) && up.initializer === cursor) {
      cursor = up.parent;
      continue;
    }
    if (ts.isArrayLiteralExpression(up) && up.elements.includes(cursor)) {
      cursor = up;
      continue;
    }
    const outer = unwrapParens(cursor, ts);
    let pattern = outer;
    while (ts.isParenthesizedExpression(pattern)) pattern = pattern.expression;
    const assign = outer.parent;
    return Boolean(
      (ts.isObjectLiteralExpression(pattern) || ts.isArrayLiteralExpression(pattern)) &&
      ts.isBinaryExpression(assign) &&
      assign.left === outer &&
      ts.isAssignmentOperator(assign.operatorToken.kind),
    );
  }
}

module.exports = { forwardedOption, guardedProof, memberWrite, passedOn };
