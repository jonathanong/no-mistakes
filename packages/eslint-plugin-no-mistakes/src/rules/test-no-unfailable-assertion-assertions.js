"use strict";

const { EXPECT_MODULES, isExpectCallee, memberName, unwrap } = require("./test-message-expect");
const { resolveVariable } = require("./test-no-shared-state-aliases");
const { moduleMember } = require("./test-no-wall-time-assertions-bindings");

const ASSERTION_MODULES = new Set([...EXPECT_MODULES, "@playwright/test"]);
const SAME_VALUE_MATCHERS = new Set(["toBe", "toEqual", "toStrictEqual"]);

function isFrameworkExpect(node, context) {
  if (
    isExpectCallee(node, context, new Set(), ASSERTION_MODULES, {
      allowForwarding: false,
      allowPoll: false,
    })
  ) {
    return true;
  }
  const member = moduleMember(unwrap(node), context);
  return member?.name === "expect" && ASSERTION_MODULES.has(member.source);
}

function parseAssertion(node, context) {
  const callee = unwrap(node.callee);
  const matcher = memberName(callee);
  if (!matcher) return null;
  let current = unwrap(callee.object);
  let negated = false;
  let asyncModifier = null;
  while (current?.type === "MemberExpression") {
    const modifier = memberName(current);
    if (modifier === "not") {
      negated = !negated;
    } else if (modifier === "resolves" || modifier === "rejects") {
      asyncModifier = modifier;
    } else {
      return null;
    }
    current = unwrap(current.object);
  }
  if (current?.type !== "CallExpression" || !isFrameworkExpect(current.callee, context)) {
    return null;
  }
  const actual = current.arguments[0];
  if (!actual || actual.type === "SpreadElement") return null;
  return {
    expectCall: current,
    actual,
    matcher,
    negated,
    asyncModifier,
    matcherArgs: node.arguments,
  };
}

function immutableConstDef(identifier, context) {
  const variable = resolveVariable(identifier, context);
  if (variable?.defs.length !== 1) return null;
  const [def] = variable.defs;
  if (
    def.type !== "Variable" ||
    def.parent.kind !== "const" ||
    !def.node.init ||
    def.node.init.range[1] > identifier.range[0] ||
    variable.references.some((ref) => ref.isWrite() && !ref.init)
  ) {
    return null;
  }
  return def.node;
}

function isConstPrimitiveLiteral(node, context) {
  const current = unwrap(node);
  if (current.type === "Literal") return true;
  if (current.type === "TemplateLiteral") return current.expressions.length === 0;
  if (
    current.type === "UnaryExpression" &&
    (current.operator === "-" || current.operator === "+")
  ) {
    const arg = unwrap(current.argument);
    return (
      arg.type === "Literal" && (typeof arg.value === "number" || typeof arg.value === "bigint")
    );
  }
  if (current.type !== "Identifier") return false;
  const variable = resolveVariable(current, context);
  if (!variable?.defs.length) return current.name === "undefined";
  const decl = immutableConstDef(current, context);
  return decl?.id.type === "Identifier" && isConstPrimitiveLiteral(decl.init, context);
}

function canonicalPath(node, context) {
  const current = unwrap(node);
  if (current.type === "Identifier") {
    return { rootVar: resolveVariable(current, context), rootName: current.name, segments: [] };
  }
  if (current.type === "ThisExpression") {
    return { rootVar: null, rootName: "this", segments: [] };
  }
  const segment = memberName(current);
  if (segment === null) return null;
  const base = canonicalPath(current.object, context);
  return base ? { ...base, segments: [...base.segments, segment] } : null;
}

function samePath(left, right) {
  if (!left || !right || left.segments.length !== right.segments.length) return false;
  if (left.rootVar || right.rootVar) {
    if (left.rootVar !== right.rootVar) return false;
  } else if (left.rootName !== right.rootName) {
    return false;
  }
  return left.segments.every((segment, index) => segment === right.segments[index]);
}

function isSameValueAssertion(assertion, context) {
  if (assertion.asyncModifier) return false;
  if (isConstPrimitiveLiteral(assertion.actual, context)) {
    return assertion.matcherArgs.every(
      (arg) => arg.type !== "SpreadElement" && isConstPrimitiveLiteral(arg, context),
    );
  }
  if (
    !assertion.negated &&
    SAME_VALUE_MATCHERS.has(assertion.matcher) &&
    assertion.matcherArgs.length === 1 &&
    assertion.matcherArgs[0].type !== "SpreadElement"
  ) {
    return samePath(
      canonicalPath(assertion.actual, context),
      canonicalPath(assertion.matcherArgs[0], context),
    );
  }
  return false;
}

module.exports = {
  canonicalPath,
  immutableConstDef,
  isSameValueAssertion,
  parseAssertion,
  samePath,
};
