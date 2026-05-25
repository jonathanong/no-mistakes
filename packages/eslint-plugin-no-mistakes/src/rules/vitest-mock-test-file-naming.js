"use strict";

const { rule } = require("../helpers");

const TEST_FILE_PATTERN = /\.(?:test|spec)\.[cm]?[jt]sx?$/;
const MOCK_TEST_FILE_PATTERN = /\.mock\.test\.[cm]?[jt]sx?$/;
const MOCK_METHODS = new Set([
  "fn",
  "mock",
  "doMock",
  "unmock",
  "doUnmock",
  "spyOn",
  "clearAllMocks",
  "resetAllMocks",
  "restoreAllMocks",
  "stubEnv",
  "stubGlobal",
  "setSystemTime",
]);

const MOCK_CHAIN_METHODS = new Set([
  "mockImplementation",
  "mockImplementationOnce",
  "mockReturnValue",
  "mockReturnValueOnce",
  "mockResolvedValue",
  "mockResolvedValueOnce",
  "mockRejectedValue",
  "mockRejectedValueOnce",
  "mockReset",
  "mockRestore",
  "mockClear",
]);

function isTestFile(filename) {
  return TEST_FILE_PATTERN.test(filename.replace(/\\/g, "/"));
}

function isMockTestFile(filename) {
  return MOCK_TEST_FILE_PATTERN.test(filename.replace(/\\/g, "/"));
}

function propertyName(node) {
  if (node.type === "Literal") return String(node.value);
  return node.name;
}

function isFrameworkBinding(node, context) {
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === node.name);
    if (!variable) {
      scope = scope.upper;
      continue;
    }
    return variable.defs.some(
      (def) =>
        (def.type === "ImportBinding" &&
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

function isMockingCall(node, context) {
  return (
    node.callee.type === "MemberExpression" &&
    node.callee.object.type === "Identifier" &&
    isFrameworkBinding(node.callee.object, context) &&
    MOCK_METHODS.has(propertyName(node.callee.property))
  );
}

module.exports = rule(
  {
    type: "problem",
    docs: { description: "require .mock.test filenames for mock-heavy tests", recommended: false },
    schema: [],
    messages: {
      needsMock: "Tests using mocking APIs must use a .mock.test.<ext> filename.",
      unnecessaryMock: "Tests without mocking APIs must not use a .mock.test.<ext> filename.",
    },
  },
  (context) => {
    let usesMocking = false;
    const mockVars = new Set();
    function isMockChainCall(node) {
      if (node.callee.type !== "MemberExpression") return false;
      if (!MOCK_CHAIN_METHODS.has(propertyName(node.callee.property))) return false;
      const obj = node.callee.object;
      if (obj.type === "Identifier") return mockVars.has(obj.name);
      return obj.type === "CallExpression" && (isMockingCall(obj, context) || isMockChainCall(obj));
    }
    return {
      VariableDeclarator(node) {
        if (
          node.id.type === "Identifier" &&
          node.init?.type === "CallExpression" &&
          isMockingCall(node.init, context)
        ) {
          mockVars.add(node.id.name);
        }
      },
      AssignmentExpression(node) {
        if (
          node.left.type === "Identifier" &&
          node.right.type === "CallExpression" &&
          isMockingCall(node.right, context)
        ) {
          mockVars.add(node.left.name);
        }
      },
      CallExpression(node) {
        if (isMockingCall(node, context) || isMockChainCall(node)) usesMocking = true;
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
);
