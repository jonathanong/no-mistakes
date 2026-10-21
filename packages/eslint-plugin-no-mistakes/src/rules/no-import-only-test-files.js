"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { literalString } = require("./module-mock-helpers");

const TEST_FILE_PATTERN = /\.(?:mock\.)?(?:test|spec)\.[cm]?[jt]sx?$/;
const TEST_IMPORT_PATTERN = /\.(?:mock\.)?(?:test|spec)(?:\.[cm]?[jt]sx?)?$/;

function isTestFile(filename) {
  return TEST_FILE_PATTERN.test(filename.replace(/\\/g, "/"));
}

function isTestImportSource(source) {
  return typeof source === "string" && TEST_IMPORT_PATTERN.test(source);
}

function stringArgument(node) {
  const source = literalString(node) ?? node?.value;
  return typeof source === "string" ? source : null;
}

function requireSource(call) {
  if (call?.type !== "CallExpression") return null;
  let callee = call.callee;
  let args = call.arguments;
  while (callee?.type === "ChainExpression") callee = callee.expression;
  if (callee?.type === "SequenceExpression") callee = callee.expressions.at(-1);
  if (callee?.type === "MemberExpression" && !callee.computed) {
    if (callee.property?.name === "call") {
      callee = callee.object;
      args = args.slice(1);
    } else if (callee.property?.name === "apply" && args[1]?.type === "ArrayExpression") {
      callee = callee.object;
      args = args[1].elements;
    }
  }
  if (callee?.type !== "Identifier" || callee.name !== "require") return null;
  return stringArgument(args[0]);
}

function isTestRequire(statement) {
  if (
    statement.type === "ExpressionStatement" &&
    isTestImportSource(requireSource(statement.expression))
  ) {
    return true;
  }
  return (
    statement.type === "VariableDeclaration" &&
    statement.declarations.length > 0 &&
    statement.declarations.every((declaration) =>
      isTestImportSource(requireSource(declaration.init)),
    )
  );
}

function isTestImport(statement) {
  return statement.type === "ImportDeclaration" && isTestImportSource(statement.source?.value);
}

module.exports = Object.assign(
  rule(
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
        const suppressed = ruleSuppression(context, "no-import-only-test-files");
        if (!isTestFile(context.filename) || node.body.length === 0) return;
        const statements = node.body.filter((statement) => statement.directive === undefined);
        if (statements.length === 0) return;
        if (
          suppressed(node) ||
          !statements.every((statement) => isTestImport(statement) || isTestRequire(statement))
        ) {
          return;
        }
        context.report({ node, messageId: "aggregate" });
      },
    }),
  ),
  { __test: { requireSource } },
);
