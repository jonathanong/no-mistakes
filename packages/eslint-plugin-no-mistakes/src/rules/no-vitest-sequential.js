"use strict";

const { rule } = require("../helpers");

const TEST_NAMES = new Set(["test", "it", "describe"]);
const TEST_FILE_PATTERN = /\.(?:test|spec)\.[cm]?[jt]sx?$/;

function isTestFile(filename) {
  return TEST_FILE_PATTERN.test(filename.replace(/\\/g, "/"));
}

function propertyName(node, computed = false) {
  if (node.type === "Literal") return String(node.value);
  if (computed) return null;
  return node.name;
}

function rootTestName(node) {
  if (node?.type === "Identifier") return node.name;
  if (node?.type === "MemberExpression") return rootTestName(node.object);
  if (node?.type === "CallExpression") return rootTestName(node.callee);
  return null;
}

function hasSequentialMember(node) {
  if (node?.type === "MemberExpression") {
    return (
      propertyName(node.property, node.computed) === "sequential" ||
      hasSequentialMember(node.object)
    );
  }
  if (node?.type === "CallExpression") return hasSequentialMember(node.callee);
  return false;
}

function hasSequentialOption(node) {
  return node.arguments
    .slice(1)
    .some(
      (argument) =>
        argument.type === "ObjectExpression" &&
        argument.properties.some(
          (property) =>
            property.type === "Property" &&
            propertyName(property.key) === "sequential" &&
            property.value.type === "Literal" &&
            property.value.value === true,
        ),
    );
}

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow Vitest sequential test modifiers", recommended: false },
    schema: [],
    messages: { sequential: "Use parallel tests instead of .sequential." },
  },
  (context) => {
    let usesVitestImport = false;
    return {
      ImportDeclaration(node) {
        if (node.source.value === "vitest") usesVitestImport = true;
      },
      CallExpression(node) {
        if (!isTestFile(context.filename) && !usesVitestImport) return;
        if (!TEST_NAMES.has(rootTestName(node.callee))) return;
        if (node.callee.type === "MemberExpression" && hasSequentialMember(node.callee)) {
          context.report({ node: node.callee, messageId: "sequential" });
          return;
        }
        if (!hasSequentialOption(node)) return;
        context.report({ node: node.callee, messageId: "sequential" });
      },
    };
  },
);
