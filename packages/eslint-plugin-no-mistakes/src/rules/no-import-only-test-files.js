"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { literalString } = require("./module-mock-helpers");
const { resolveVariable } = require("./test-no-shared-state-aliases");

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

function forwardedName(member) {
  if (member?.type !== "MemberExpression") return null;
  if (member.computed && member.property?.type === "Identifier") return null;
  if (!member.computed && member.property?.type === "Identifier") return member.property.name;
  return literalString(member.property);
}

function requireBinding(node, context, seen = new Set()) {
  if (node?.type !== "Identifier") return false;
  if (!context) return node.name === "require";
  const variable = resolveVariable(node, context);
  if (!variable || variable.defs.length === 0) return node.name === "require";
  if (seen.has(variable)) return false;
  seen.add(variable);
  const def = variable.defs.length === 1 ? variable.defs[0] : null;
  return (
    def?.type === "Variable" &&
    def.node?.parent?.kind === "const" &&
    def.node.id.type === "Identifier" &&
    !variable.references.some((ref) => ref.isWrite() && !ref.init) &&
    requireBinding(def.node.init, context, seen)
  );
}

function isRequireAlias(statement, context) {
  return (
    statement.type === "VariableDeclaration" &&
    statement.kind === "const" &&
    statement.declarations.length > 0 &&
    statement.declarations.every(
      (declaration) =>
        declaration.id.type === "Identifier" && requireBinding(declaration.id, context),
    )
  );
}

function requireSource(call, context) {
  let current = call;
  while (current?.type === "ChainExpression") current = current.expression;
  if (current?.type !== "CallExpression") return null;
  let callee = current.callee;
  let args = current.arguments;
  while (callee?.type === "ChainExpression") callee = callee.expression;
  if (callee?.type === "SequenceExpression") callee = callee.expressions.at(-1);
  const name = forwardedName(callee);
  if (name === "call") {
    callee = callee.object;
    args = args.slice(1);
  } else if (name === "apply" && args[1]?.type === "ArrayExpression") {
    callee = callee.object;
    args = args[1].elements;
  }
  while (callee?.type === "ChainExpression") callee = callee.expression;
  if (!requireBinding(callee, context)) return null;
  return stringArgument(args?.[0]);
}

function isTestRequire(statement, context) {
  if (
    statement.type === "ExpressionStatement" &&
    isTestImportSource(requireSource(statement.expression, context))
  ) {
    return true;
  }
  return (
    statement.type === "VariableDeclaration" &&
    statement.declarations.length > 0 &&
    statement.declarations.every((declaration) =>
      isTestImportSource(requireSource(declaration.init, context)),
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
        const statements = node.body.filter((statement) => typeof statement.directive !== "string");
        if (statements.length === 0) return;
        if (
          suppressed(node) ||
          statements.some((statement) => suppressed(statement)) ||
          !statements.some(
            (statement) => isTestImport(statement) || isTestRequire(statement, context),
          ) ||
          !statements.every(
            (statement) =>
              isTestImport(statement) ||
              isTestRequire(statement, context) ||
              isRequireAlias(statement, context),
          )
        ) {
          return;
        }
        context.report({ node, messageId: "aggregate" });
      },
    }),
  ),
  { __test: { requireSource } },
);
