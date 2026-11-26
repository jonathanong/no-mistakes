"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression } = require("./async-ast");
const { literalString, memberPropertyName } = require("./module-mock-helpers");
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
  const current = unwrapExpression(node);
  const source = literalString(current) ?? current?.value;
  return typeof source === "string" ? source : null;
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
    requireBinding(unwrapExpression(def.node.init), context, seen)
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
  const current = unwrapExpression(call);
  if (current?.type !== "CallExpression") return null;
  let callee = unwrapExpression(current.callee);
  let args = current.arguments;
  if (callee?.type === "SequenceExpression") callee = unwrapExpression(callee.expressions.at(-1));
  const name = callee?.type === "MemberExpression" ? memberPropertyName(callee) : null;
  if (name === "call") {
    callee = unwrapExpression(callee.object);
    args = args.slice(1);
  } else if (name === "apply") {
    const list = unwrapExpression(args?.[1]);
    if (list?.type !== "ArrayExpression") return null;
    callee = unwrapExpression(callee.object);
    args = list.elements;
  }
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

function importEqualsSource(node) {
  if (node?.type !== "TSImportEqualsDeclaration" || node.importKind === "type") return null;
  const ref = node.moduleReference;
  if (ref?.type !== "TSExternalModuleReference") return null;
  return stringArgument(ref.expression);
}

function isTestImport(statement) {
  if (statement.type === "ImportDeclaration" || statement.type === "ExportAllDeclaration") {
    return isTestImportSource(statement.source?.value);
  }
  if (statement.type === "ExportNamedDeclaration" && statement.source) {
    return isTestImportSource(statement.source.value);
  }
  if (statement.type === "TSImportEqualsDeclaration")
    return isTestImportSource(importEqualsSource(statement));
  return false;
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
