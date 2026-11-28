"use strict";

const { unwrap } = require("./test-message-expect");
const { resolveVariable } = require("./test-no-shared-state-aliases");
const {
  canonicalPath,
  immutableConstDef,
  parseAssertion,
  samePath,
} = require("./test-no-unfailable-assertion-assertions");

const OPTIONAL_ALLOWS_UNDEFINED = new Set(["toBeUndefined", "toBeFalsy", "toBeNull"]);
const EQUALITY_MATCHERS = new Set(["toBe", "toEqual", "toStrictEqual"]);

function unwrapStatementCall(statement) {
  if (statement.type !== "ExpressionStatement") return null;
  let expr = unwrap(statement.expression);
  if (expr.type === "AwaitExpression") expr = unwrap(expr.argument);
  return expr.type === "CallExpression" ? expr : null;
}

function isDefinedMatcher(assertion) {
  return (
    !assertion.asyncModifier &&
    ((!assertion.negated && assertion.matcher === "toBeDefined") ||
      (assertion.negated && assertion.matcher === "toBeUndefined"))
  );
}

function isUndefinedLiteral(node, context) {
  const current = unwrap(node);
  return (
    current.type === "Identifier" &&
    current.name === "undefined" &&
    !resolveVariable(current, context)?.defs.length
  );
}

function optionalAssertionFailsOnUndefined(assertion, context) {
  if (assertion.negated) return assertion.matcher === "toBeUndefined";
  if (OPTIONAL_ALLOWS_UNDEFINED.has(assertion.matcher)) return false;
  if (
    EQUALITY_MATCHERS.has(assertion.matcher) &&
    assertion.matcherArgs.length === 1 &&
    isUndefinedLiteral(assertion.matcherArgs[0], context)
  ) {
    return false;
  }
  return true;
}

function derefKind(node, target, context) {
  if (node.type === "ChainExpression") return derefKind(node.expression, target, context);
  if (node.type === "AwaitExpression") return derefKind(node.argument, target, context);
  if (node.type === "Identifier") {
    const decl = immutableConstDef(node, context);
    return decl?.id.type === "Identifier" ? derefKind(decl.init, target, context) : null;
  }
  if (node.type === "MemberExpression" || node.type === "CallExpression") {
    const base = node.type === "MemberExpression" ? node.object : node.callee;
    if (
      base.type === "TSNonNullExpression" &&
      samePath(canonicalPath(base.expression, context), target)
    ) {
      return "non-null";
    }
    if (node.optional && samePath(canonicalPath(base, context), target)) return "optional";
    return derefKind(base, target, context);
  }
  return null;
}

function writesRootVariable(statement, rootVar) {
  if (!rootVar) return false;
  const [start, end] = statement.range;
  return rootVar.references.some(
    (ref) =>
      ref.isWrite() &&
      !ref.init &&
      ref.identifier.range[0] >= start &&
      ref.identifier.range[1] <= end,
  );
}

function findDefinedThenDeref(statements, context, reportedAssertions) {
  const results = [];
  for (let index = 0; index < statements.length; index++) {
    const call = unwrapStatementCall(statements[index]);
    if (!call || reportedAssertions.has(call)) continue;
    const assertion = parseAssertion(call, context);
    if (!assertion || !isDefinedMatcher(assertion)) continue;
    const target = canonicalPath(assertion.actual, context);
    if (!target) continue;
    for (let next = index + 1; next < statements.length; next++) {
      const sibling = statements[next];
      if (writesRootVariable(sibling, target.rootVar)) break;
      const nextCall = unwrapStatementCall(sibling);
      const nextAssertion = nextCall && parseAssertion(nextCall, context);
      if (!nextAssertion || isDefinedMatcher(nextAssertion)) continue;
      const kind = derefKind(nextAssertion.actual, target, context);
      if (
        kind === "non-null" ||
        (kind === "optional" && optionalAssertionFailsOnUndefined(nextAssertion, context))
      ) {
        results.push(call);
        break;
      }
    }
  }
  return results;
}

module.exports = { findDefinedThenDeref };
