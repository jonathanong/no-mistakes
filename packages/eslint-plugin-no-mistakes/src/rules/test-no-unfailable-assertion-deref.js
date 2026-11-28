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
const CONTROL_FLOW_STATEMENTS = new Set([
  "BreakStatement",
  "ContinueStatement",
  "DoWhileStatement",
  "ForInStatement",
  "ForOfStatement",
  "ForStatement",
  "IfStatement",
  "ReturnStatement",
  "SwitchStatement",
  "ThrowStatement",
  "TryStatement",
  "WhileStatement",
]);

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
  if (current.type !== "Identifier") return false;
  const variable = resolveVariable(current, context);
  if (!variable?.defs.length) return current.name === "undefined";
  const decl = immutableConstDef(current, context);
  return decl?.id.type === "Identifier" && isUndefinedLiteral(decl.init, context);
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

function derefKind(node, target, context, afterPos) {
  if (node.type === "ChainExpression") return derefKind(node.expression, target, context, afterPos);
  if (node.type === "AwaitExpression") return derefKind(node.argument, target, context, afterPos);
  if (node.type === "Identifier") {
    const decl = immutableConstDef(node, context);
    return decl?.id.type === "Identifier" && decl.init.range[0] > afterPos
      ? derefKind(decl.init, target, context, afterPos)
      : null;
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
    return derefKind(base, target, context, afterPos);
  }
  return null;
}

function pathPrefixMatches(prefix, full) {
  if (!prefix || prefix.segments.length > full.segments.length) return false;
  if (prefix.rootVar || full.rootVar) {
    if (prefix.rootVar !== full.rootVar) return false;
  } else if (prefix.rootName !== full.rootName) {
    return false;
  }
  return prefix.segments.every((segment, index) => segment === full.segments[index]);
}

function writesTarget(statement, target, context) {
  const [start, end] = statement.range;
  if (
    target.rootVar?.references.some(
      (ref) =>
        ref.isWrite() &&
        !ref.init &&
        ref.identifier.range[0] >= start &&
        ref.identifier.range[1] <= end,
    )
  ) {
    return true;
  }
  if (statement.type !== "ExpressionStatement") return false;
  const expr = unwrap(statement.expression);
  const mutated =
    expr.type === "AssignmentExpression"
      ? expr.left
      : expr.type === "UnaryExpression" && expr.operator === "delete"
        ? expr.argument
        : null;
  return Boolean(mutated && pathPrefixMatches(canonicalPath(mutated, context), target));
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
      if (CONTROL_FLOW_STATEMENTS.has(sibling.type) || writesTarget(sibling, target, context)) {
        break;
      }
      const nextCall = unwrapStatementCall(sibling);
      const nextAssertion = nextCall && parseAssertion(nextCall, context);
      if (!nextAssertion || isDefinedMatcher(nextAssertion)) continue;
      const kind = derefKind(nextAssertion.actual, target, context, call.range[1]);
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
