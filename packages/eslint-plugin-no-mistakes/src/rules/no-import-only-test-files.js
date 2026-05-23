"use strict";

const { rule } = require("../helpers");

const TEST_FILE_PATTERN = /\.(?:mock\.)?(?:test|spec)\.[cm]?[jt]sx?$/;
const TEST_IMPORT_PATTERN = /\.(?:mock\.)?(?:test|spec)(?:\.[cm]?[jt]sx?)?$/;

function isTestFile(filename) {
  return TEST_FILE_PATTERN.test(filename.replace(/\\/g, "/"));
}

function isTestImportSource(source) {
  return typeof source === "string" && TEST_IMPORT_PATTERN.test(source);
}

function isTestRequire(statement) {
  return (
    statement.type === "ExpressionStatement" &&
    statement.expression.type === "CallExpression" &&
    statement.expression.callee.type === "Identifier" &&
    statement.expression.callee.name === "require" &&
    isTestImportSource(statement.expression.arguments[0]?.value)
  );
}

function isTestImport(statement) {
  return statement.type === "ImportDeclaration" && isTestImportSource(statement.source?.value);
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow aggregate test files that only import tests",
      recommended: false,
    },
    schema: [],
    messages: {
      aggregate:
        "Do not create aggregate test files that only import other test files; let the test runner discover those files directly.",
    },
  },
  (context) => ({
    Program(node) {
      if (!isTestFile(context.filename) || node.body.length === 0) return;
      if (!node.body.every((statement) => isTestImport(statement) || isTestRequire(statement)))
        return;
      context.report({ node, messageId: "aggregate" });
    },
  }),
);
