"use strict";

const { posix, relative } = require("node:path");
const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { isUnresolved, moduleLoad } = require("./module-load");
const { reportUnanalyzable, UNANALYZABLE_SCHEMA } = require("./rule-unanalyzable");
const {
  EXPECT_MODULES,
  isExpectCallee,
  isNamespace,
  memberName,
  unwrap,
} = require("./test-message-expect");
const { resolveVariable } = require("./test-no-shared-state-aliases");
const { localOrigin, moduleMember } = require("./test-no-wall-time-assertions-bindings");

const TEST_FILE = /\.(?:test|spec)\.[cm]?[jt]sx?$/;
const ASSERTION_MODULES = new Set([...EXPECT_MODULES, "@playwright/test"]);
const TIMER_MODULES = new Map([
  ["vi", new Set(["vitest"])],
  ["jest", new Set(["@jest/globals"])],
]);
const CLOCKS = new Set(["Date", "performance"]);
const PERFORMANCE_MODULES = new Set(["node:perf_hooks", "perf_hooks"]);
const MATCHERS = new Set([
  "toBeLessThan",
  "toBeLessThanOrEqual",
  "toBeGreaterThan",
  "toBeGreaterThanOrEqual",
]);

function isClockValue(node, context) {
  const value = localOrigin(node, context);
  if (value?.type !== "CallExpression") return false;
  const callee = unwrap(value.callee);
  if (memberName(callee) !== "now") return false;
  const clock = unwrap(callee.object);
  if (
    clock?.type === "Identifier" &&
    CLOCKS.has(clock.name) &&
    !resolveVariable(clock, context)?.defs.length
  ) {
    return true;
  }
  const member = moduleMember(clock, context);
  return member?.name === "performance" && PERFORMANCE_MODULES.has(member.source);
}

function isElapsedDifference(node, context) {
  const value = localOrigin(node, context);
  return (
    value?.type === "BinaryExpression" &&
    value.operator === "-" &&
    (isClockValue(value.left, context) || isClockValue(value.right, context))
  );
}

function isTimerObject(node, context) {
  const object = unwrap(node);
  if (
    object?.type === "Identifier" &&
    TIMER_MODULES.has(object.name) &&
    !resolveVariable(object, context)?.defs.length
  ) {
    return true;
  }
  const member = moduleMember(object, context);
  if (member && TIMER_MODULES.get(member.name)?.has(member.source)) return true;
  const modules = TIMER_MODULES.get(memberName(object));
  return modules && isNamespace(unwrap(object.object), context, new Set(), modules);
}

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

function opaqueNow(node, context) {
  const value = localOrigin(node, context);
  if (value?.type !== "CallExpression") return false;
  const callee = unwrap(value.callee);
  if (callee?.type === "MemberExpression" && callee.computed) return false;
  if (memberName(callee) !== "now") return false;
  return isUnresolved(moduleLoad(unwrap(callee.object), context));
}

function opaqueWallClock(node, context) {
  const callee = unwrap(node.callee);
  if (callee?.computed || !MATCHERS.has(memberName(callee))) return false;
  let assertion = unwrap(callee.object);
  while (memberName(assertion) === "not") assertion = unwrap(assertion.object);
  if (assertion?.type !== "CallExpression" || !isFrameworkExpect(assertion.callee, context)) {
    return false;
  }
  const argument = localOrigin(assertion.arguments?.[0], context);
  return (
    argument?.type === "BinaryExpression" &&
    argument.operator === "-" &&
    (opaqueNow(argument.left, context) || opaqueNow(argument.right, context))
  );
}

function isWallTimeAssertion(callee, context) {
  if (callee?.computed || !MATCHERS.has(memberName(callee))) return false;
  let assertion = unwrap(callee.object);
  while (memberName(assertion) === "not") assertion = unwrap(assertion.object);
  return (
    assertion?.type === "CallExpression" &&
    isFrameworkExpect(assertion.callee, context) &&
    isElapsedDifference(assertion.arguments[0], context)
  );
}

function isAllowedPath(context) {
  return context.options[0]?.allowedPaths?.some((glob) =>
    posix.matchesGlob(relative(context.cwd, context.filename).replace(/\\/g, "/"), glob),
  );
}

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow asserting on elapsed wall time in tests", recommended: false },
    schema: [
      {
        type: "object",
        properties: {
          allowedPaths: { type: "array", items: { type: "string" } },
          unanalyzable: UNANALYZABLE_SCHEMA,
        },
        additionalProperties: false,
      },
    ],
    messages: {
      wallTime:
        "Avoid asserting on elapsed wall time. Assert on observable behavior or control time with fake timers.",
      unanalyzable:
        "Cannot tell whether {{subject}} reads wall time. Use a static Date, performance, or perf_hooks binding.",
    },
  },
  (context) => {
    if (!TEST_FILE.test(context.filename) || isAllowedPath(context)) return {};
    const suppressed = ruleSuppression(context, "test-no-wall-time-assertions");
    let usesFakeTimers = false;
    const assertions = [];
    const advisories = [];
    return {
      CallExpression(node) {
        if (usesFakeTimers) return;
        const callee = unwrap(node.callee);
        if (memberName(callee) === "useFakeTimers" && isTimerObject(callee.object, context)) {
          usesFakeTimers = true;
        } else if (isWallTimeAssertion(callee, context) && !suppressed(node)) {
          assertions.push(node);
        } else if (!suppressed(node) && opaqueWallClock(node, context)) {
          advisories.push(node);
        }
      },
      "Program:exit"() {
        if (usesFakeTimers) return;
        for (const node of advisories) {
          reportUnanalyzable(context, node, "this elapsed-time assertion");
        }
        for (const node of assertions) context.report({ node, messageId: "wallTime" });
      },
    };
  },
);
