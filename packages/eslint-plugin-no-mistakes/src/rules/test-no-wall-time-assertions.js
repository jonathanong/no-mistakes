"use strict";

const { posix, relative } = require("node:path");
const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { requireSource } = require("./async-target-sources");
const { literalString } = require("./module-mock-helpers");
const {
  EXPECT_MODULES,
  isExpectCallee,
  isNamespace,
  memberName,
  propertyName,
  unwrap,
} = require("./test-message-expect");
const { resolveVariable } = require("./test-no-shared-state-aliases");
const { importSpecifierName } = require("./test-no-shared-state-callees");

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

function constDeclaration(identifier, variable) {
  if (variable?.defs.length !== 1) return null;
  const [definition] = variable.defs;
  const declarator = definition.node;
  return definition.type === "Variable" &&
    definition.parent.kind === "const" &&
    declarator.init &&
    declarator.init.range[1] <= identifier.range[0] &&
    !variable.references.some((reference) => reference.isWrite() && !reference.init)
    ? declarator
    : null;
}

function localOrigin(node, context) {
  let current = unwrap(node);
  while (current?.type === "Identifier") {
    const variable = resolveVariable(current, context);
    const declarator = constDeclaration(current, variable);
    if (
      declarator?.id.type !== "Identifier" ||
      context.sourceCode.getScope(current).variableScope !== variable.scope.variableScope
    ) {
      return current;
    }
    current = unwrap(declarator.init);
  }
  return current;
}

function requiredNamespace(node, context) {
  const value = unwrap(node);
  if (value?.type !== "Identifier") return requireSource(value, context);
  const declaration = constDeclaration(value, resolveVariable(value, context));
  return declaration?.id.type === "Identifier" ? requireSource(declaration.init, context) : null;
}

function requiredMemberAccess(node, context) {
  const name = memberName(node);
  const source = name && requiredNamespace(node.object, context);
  return source ? { name, source } : null;
}

function moduleMember(node, context) {
  if (node?.type !== "Identifier") return requiredMemberAccess(node, context);
  const variable = resolveVariable(node, context);
  const definition = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (
    definition?.type === "ImportBinding" &&
    definition.node.type === "ImportSpecifier" &&
    definition.node.importKind !== "type" &&
    definition.parent.importKind !== "type"
  ) {
    return {
      name: importSpecifierName(definition.node),
      source: definition.parent.source.value,
    };
  }
  const declaration = constDeclaration(node, variable);
  if (declaration?.id.type === "Identifier") {
    return requiredMemberAccess(unwrap(declaration.init), context);
  }
  if (declaration?.id.type !== "ObjectPattern") return null;
  const source = requiredNamespace(declaration.init, context);
  if (!source) return null;
  const property = declaration.id.properties.find(
    (item) => item.type === "Property" && item.value.name === node.name,
  );
  const name = property?.computed ? literalString(property.key) : propertyName(property?.key);
  return name ? { name, source } : null;
}

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

function isWallTimeAssertion(callee, context) {
  if (!MATCHERS.has(memberName(callee))) return false;
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
        properties: { allowedPaths: { type: "array", items: { type: "string" } } },
        additionalProperties: false,
      },
    ],
    messages: {
      wallTime:
        "Avoid asserting on elapsed wall time. Assert on observable behavior or control time with fake timers.",
    },
  },
  (context) => {
    if (!TEST_FILE.test(context.filename) || isAllowedPath(context)) return {};
    const suppressed = ruleSuppression(context, "test-no-wall-time-assertions");
    let usesFakeTimers = false;
    const assertions = [];
    return {
      CallExpression(node) {
        if (usesFakeTimers) return;
        const callee = unwrap(node.callee);
        if (memberName(callee) === "useFakeTimers" && isTimerObject(callee.object, context)) {
          usesFakeTimers = true;
        } else if (isWallTimeAssertion(callee, context) && !suppressed(node)) {
          assertions.push(node);
        }
      },
      "Program:exit"() {
        if (usesFakeTimers) return;
        for (const node of assertions) context.report({ node, messageId: "wallTime" });
      },
    };
  },
);
