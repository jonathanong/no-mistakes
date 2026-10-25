"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { literalString } = require("./module-mock-helpers");
const { resolveVariable } = require("./test-no-shared-state-aliases");

const TEST_FILE_PATTERN = /\.(?:test|spec)\.[cm]?[jt]sx?$/;
const MOCK_TEST_FILE_PATTERN = /\.mock\.test\.[cm]?[jt]sx?$/;
// Only module-level mocking requires a `.mock.test` filename. `vi.fn()`,
// `vi.spyOn`, env stubs, and `.mock*()` chain helpers are test doubles, not
// module mocks (issue #388).
const MOCK_METHODS = new Set(["mock", "doMock", "unmock", "doUnmock"]);

function isTestFile(filename) {
  return TEST_FILE_PATTERN.test(filename.replace(/\\/g, "/"));
}

function isMockTestFile(filename) {
  return MOCK_TEST_FILE_PATTERN.test(filename.replace(/\\/g, "/"));
}

function propertyName(node, computed) {
  if (!node) return null;
  if (computed && node.type === "Identifier") return null;
  if (!computed && typeof node.name === "string" && node.type !== "Literal") return node.name;
  return literalString(node) || (node.type === "Literal" ? String(node.value) : null);
}

function unwrap(node) {
  while (node?.type === "ChainExpression") node = node.expression;
  if (node?.type === "SequenceExpression" && node.expressions?.length) {
    return unwrap(node.expressions.at(-1));
  }
  return node;
}

function isFrameworkBinding(node, context, seen = new Set()) {
  node = unwrap(node);
  if (
    node?.type === "MemberExpression" &&
    ["vi", "jest"].includes(propertyName(node.property, node.computed))
  )
    return isFrameworkBinding(node.object, context, seen);
  if (node?.type !== "Identifier" || seen.has(node.name)) return false;
  seen.add(node.name);
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === node.name);
    if (!variable) {
      scope = scope.upper;
      continue;
    }
    const def = variable.defs.length === 1 ? variable.defs[0] : null;
    if (
      def?.type === "Variable" &&
      def.node?.parent?.kind === "const" &&
      def.node.init &&
      !variable.references.some((ref) => ref.isWrite() && !ref.init)
    ) {
      if (
        def.node.id.type === "Identifier" &&
        ["Identifier", "MemberExpression"].includes(unwrap(def.node.init)?.type)
      )
        return isFrameworkBinding(def.node.init, context, seen);
      if (
        def.node.id.type === "ObjectPattern" &&
        def.node.init.type === "Identifier" &&
        def.node.id.properties.some(
          (property) =>
            property.type === "Property" &&
            property.value?.type === "Identifier" &&
            property.value.name === node.name &&
            ["vi", "jest"].includes(propertyName(property.key, property.computed)),
        )
      )
        return isFrameworkBinding(def.node.init, context, seen);
    }
    return variable.defs.some(
      (def) =>
        (def.type === "ImportBinding" &&
          def.node.type !== "ImportDefaultSpecifier" &&
          (def.parent?.source?.value === "vitest" ||
            def.parent?.source?.value === "@jest/globals")) ||
        isFrameworkRequire(def),
    );
  }
  return node.name === "vi" || node.name === "jest";
}

function isFrameworkRequire(def) {
  const init = def.node?.init;
  return (
    def.type === "Variable" &&
    init?.type === "CallExpression" &&
    init.callee.type === "Identifier" &&
    init.callee.name === "require" &&
    (init.arguments[0]?.value === "vitest" || init.arguments[0]?.value === "@jest/globals")
  );
}

function mockMember(node, context, seen = new Set()) {
  const callee = unwrap(node);
  if (callee?.type === "Identifier") {
    if (seen.has(callee.name)) return false;
    const variable = resolveVariable(callee, context);
    const def = variable?.defs.length === 1 ? variable.defs[0] : null;
    if (def?.type !== "Variable" || def.node?.parent?.kind !== "const") return false;
    seen.add(callee.name);
    if (def.node.id.type === "Identifier" && def.node.init) {
      return mockMember(def.node.init, context, seen);
    }
    if (def.node.id?.type !== "ObjectPattern" || !def.node.init) return false;
    return def.node.id.properties.some((property) => {
      if (property.type !== "Property") return false;
      let value = property.value;
      while (value?.type === "AssignmentPattern") value = value.left;
      const key = property.computed
        ? literalString(property.key)
        : propertyName(property.key, false);
      return (
        value?.type === "Identifier" &&
        value.name === callee.name &&
        MOCK_METHODS.has(key) &&
        isFrameworkBinding(def.node.init, context)
      );
    });
  }
  if (callee?.type !== "MemberExpression") return false;
  const method = propertyName(callee.property, callee.computed);
  if (method === "call" || method === "apply") {
    return mockMember(callee.object, context, seen);
  }
  if (!MOCK_METHODS.has(method)) return false;
  const object = unwrap(callee.object);
  if (object?.type === "Identifier") return isFrameworkBinding(object, context);
  if (object?.type !== "MemberExpression") return false;
  const inner = propertyName(object.property, object.computed);
  return (
    (inner === "vi" || inner === "jest") &&
    object.object?.type === "Identifier" &&
    isFrameworkBinding(object.object, context)
  );
}

function isMockingCall(node, context) {
  return mockMember(node.callee, context);
}

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: {
        description: "require .mock.test filenames for module-mocking tests",
        recommended: false,
      },
      schema: [],
      messages: {
        needsMock:
          "Tests that mock modules (vi.mock/vi.doMock) must use a .mock.test.<ext> filename. vi.fn()/vi.spyOn stubs do not count.",
        unnecessaryMock:
          "Tests without module mocking (vi.mock/vi.doMock) must not use a .mock.test.<ext> filename.",
      },
    },
    (context) => {
      const suppressed = ruleSuppression(context, "vitest-mock-test-file-naming");
      let usesMocking = false;
      return {
        CallExpression(node) {
          if (isMockingCall(node, context) && !suppressed(node)) usesMocking = true;
        },
        "Program:exit"(node) {
          if (!isTestFile(context.filename)) return;
          if (usesMocking && !isMockTestFile(context.filename)) {
            context.report({ node, messageId: "needsMock" });
          }
          if (!usesMocking && isMockTestFile(context.filename)) {
            context.report({ node, messageId: "unnecessaryMock" });
          }
        },
      };
    },
  ),
  { __test: { mockMember, propertyName } },
);
