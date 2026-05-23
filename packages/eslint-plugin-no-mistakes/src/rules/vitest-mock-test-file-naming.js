"use strict";

const { rule } = require("../helpers");

const TEST_FILE_PATTERN = /\.(?:test|spec)\.[cm]?[jt]sx?$/;
const MOCK_TEST_FILE_PATTERN = /\.mock\.test\.[cm]?[jt]sx?$/;
const MOCK_METHODS = new Set([
  "mock",
  "doMock",
  "unmock",
  "doUnmock",
  "spyOn",
  "fn",
  "clearAllMocks",
  "resetAllMocks",
  "restoreAllMocks",
  "stubEnv",
  "setSystemTime",
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

function isMockingCall(node) {
  return (
    node.callee.type === "MemberExpression" &&
    node.callee.object.type === "Identifier" &&
    (node.callee.object.name === "vi" || node.callee.object.name === "jest") &&
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
    return {
      CallExpression(node) {
        if (isMockingCall(node)) usesMocking = true;
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
