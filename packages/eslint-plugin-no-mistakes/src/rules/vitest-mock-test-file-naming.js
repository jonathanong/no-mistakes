"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression, unwrapSequence } = require("./async-ast");
const { literalString } = require("./logical-call");
const { frameworkBindingOpaque, isFrameworkBinding } = require("./module-mock-framework");
const { reportUnanalyzable, UNANALYZABLE_SCHEMA } = require("./rule-unanalyzable");
const { resolveVariable } = require("./test-no-shared-state-aliases");

const TEST_FILE_PATTERN = /\.(?:test|spec)\.[cm]?[jt]sx?$/;
const MOCK_TEST_FILE_PATTERN = /\.mock\.test\.[cm]?[jt]sx?$/;
// Only module-level mocking requires a `.mock.test` filename. `vi.fn()`,
// `vi.spyOn`, env stubs, and `.mock*()` chain helpers are test doubles, not
// module mocks (issue #388).
const MOCK_METHODS = new Set(["mock", "doMock", "unmock", "doUnmock"]);

function isDefaultImport(node, context, seen = new Set()) {
  let current = unwrapExpression(node);
  while (current?.type === "Identifier") {
    const variable = resolveVariable(current, context);
    if (!variable || seen.has(variable)) return false;
    const def = variable.defs.length === 1 ? variable.defs[0] : null;
    if (def?.node?.type === "ImportDefaultSpecifier") return true;
    if (
      def?.type !== "Variable" ||
      def.node?.parent?.kind !== "const" ||
      def.node.id?.type !== "Identifier" ||
      !def.node.init
    ) {
      return false;
    }
    seen.add(variable);
    current = unwrapExpression(def.node.init);
  }
  return false;
}

function isTestFile(filename) {
  return TEST_FILE_PATTERN.test(filename.replace(/\\/g, "/"));
}

function isMockTestFile(filename) {
  return MOCK_TEST_FILE_PATTERN.test(filename.replace(/\\/g, "/"));
}

function propertyName(node, computed) {
  node = unwrapExpression(node);
  if (!node) return null;
  if (computed && node.type === "Identifier") return null;
  if (!computed && typeof node.name === "string" && node.type !== "Literal") return node.name;
  return literalString(node) || (node.type === "Literal" ? String(node.value) : null);
}

function patternMock(pattern, name) {
  return pattern.properties.some((property) => {
    if (property.type !== "Property") return false;
    let value = property.value;
    while (value?.type === "AssignmentPattern") value = value.left;
    const key = property.computed ? literalString(property.key) : propertyName(property.key, false);
    return value?.type === "Identifier" && value.name === name && MOCK_METHODS.has(key);
  });
}

function destructuredOpaqueMock(node, context) {
  const callee = unwrapSequence(node.callee);
  if (callee?.type !== "Identifier") return false;
  const variable = resolveVariable(callee, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  const init = unwrapSequence(def?.node?.init);
  if (
    def?.type !== "Variable" ||
    def.node?.parent?.kind !== "const" ||
    def.node?.id?.type !== "ObjectPattern" ||
    init?.type !== "Identifier" ||
    isFrameworkBinding(init, context) ||
    !frameworkBindingOpaque(init, context)
  ) {
    return false;
  }
  return patternMock(def.node.id, callee.name);
}

function mockMember(node, context, seen = new Set()) {
  const callee = unwrapSequence(node);
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
    if (isDefaultImport(def.node.init, context)) return false;
    return patternMock(def.node.id, callee.name) && isFrameworkBinding(def.node.init, context);
  }
  if (callee?.type !== "MemberExpression") return false;
  const method = propertyName(callee.property, callee.computed);
  if (method === "call" || method === "apply") {
    return mockMember(callee.object, context, seen);
  }
  if (!MOCK_METHODS.has(method)) return false;
  const object = unwrapSequence(callee.object);
  if (object?.type === "Identifier") {
    return isFrameworkBinding(object, context) && !isDefaultImport(object, context);
  }
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

function constAliasTarget(node, context, seen) {
  const current = unwrapSequence(node);
  if (current?.type !== "Identifier") return current;
  const variable = resolveVariable(current, context);
  if (!variable || seen.has(variable) || variable.defs.length !== 1) return null;
  const def = variable.defs[0];
  if (
    def.type !== "Variable" ||
    def.node?.parent?.kind !== "const" ||
    def.node.id?.type !== "Identifier" ||
    !def.node.init
  ) {
    return null;
  }
  seen.add(variable);
  return unwrapSequence(def.node.init);
}

function opaqueCallee(node, context) {
  let callee = unwrapSequence(node);
  const seen = new Set();
  while (callee?.type === "Identifier") callee = constAliasTarget(callee, context, seen);
  while (
    callee?.type === "MemberExpression" &&
    (propertyName(callee.property, callee.computed) === "call" ||
      propertyName(callee.property, callee.computed) === "apply")
  ) {
    callee = constAliasTarget(unwrapSequence(callee.object), context, seen);
    while (callee?.type === "Identifier") callee = constAliasTarget(callee, context, seen);
  }
  return callee;
}

function opaqueMock(node, context) {
  const callee = opaqueCallee(node.callee, context);
  if (callee?.type !== "MemberExpression") return false;
  const method = propertyName(callee.property, callee.computed);
  if (method === null || !MOCK_METHODS.has(method)) return false;
  if (isFrameworkBinding(callee.object, context)) return false;
  return frameworkBindingOpaque(callee.object, context);
}

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: {
        description: "require .mock.test filenames for module-mocking tests",
        recommended: false,
      },
      schema: [
        {
          type: "object",
          additionalProperties: false,
          properties: { unanalyzable: UNANALYZABLE_SCHEMA },
        },
      ],
      messages: {
        needsMock:
          "Tests that mock modules (vi.mock/vi.doMock) must use a .mock.test.<ext> filename. vi.fn()/vi.spyOn stubs do not count.",
        unnecessaryMock:
          "Tests without module mocking (vi.mock/vi.doMock) must not use a .mock.test.<ext> filename.",
        unanalyzable:
          "Cannot tell whether {{subject}} mocks a module. Use a static vi.mock or jest.mock call.",
      },
    },
    (context) => {
      const suppressed = ruleSuppression(context, "vitest-mock-test-file-naming");
      let usesMocking = false;
      let opaque = false;
      return {
        CallExpression(node) {
          if (suppressed(node)) return;
          if (isMockingCall(node, context)) usesMocking = true;
          else if (opaqueMock(node, context) || destructuredOpaqueMock(node, context))
            opaque = true;
        },
        "Program:exit"(node) {
          if (!isTestFile(context.filename)) return;
          if (opaque) reportUnanalyzable(context, node, "a module mock");
          if (opaque && !usesMocking) return;
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
