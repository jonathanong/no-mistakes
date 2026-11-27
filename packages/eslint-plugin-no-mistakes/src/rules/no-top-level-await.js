"use strict";
const { ruleSuppression } = require("../rule-suppression");

const GUARD_WRAPPERS = new Set([
  "TSAsExpression",
  "TSTypeAssertion",
  "TSNonNullExpression",
  "TSSatisfiesExpression",
  "TSInstantiationExpression",
  "ParenthesizedExpression",
]);

function unwrapGuard(node) {
  while (GUARD_WRAPPERS.has(node?.type)) node = node.expression;
  return node;
}

function isMainGuard(node) {
  node = unwrapGuard(node);
  const object = unwrapGuard(node?.object);
  return (
    node?.type === "MemberExpression" &&
    !node.computed &&
    node.property.name === "main" &&
    object?.type === "MetaProperty" &&
    object.meta.name === "import" &&
    object.property.name === "meta"
  );
}

module.exports = {
  meta: {
    type: "problem",
    docs: {
      description: "keep asynchronous initialization out of module evaluation",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          allowInImportMetaMain: { type: "boolean" },
          message: { type: "string", pattern: "\\S" },
        },
        additionalProperties: false,
      },
    ],
    messages: { topLevelAwait: "{{message}}" },
  },
  create(context) {
    const options = context.options[0] || {};
    const suppressed = ruleSuppression(context, "no-top-level-await");
    function check(node) {
      let guarded = false;
      for (let child = node, parent = node.parent; parent; child = parent, parent = parent.parent) {
        if (
          ["FunctionDeclaration", "FunctionExpression", "ArrowFunctionExpression"].includes(
            parent.type,
          )
        )
          return;
        if (
          parent.type === "IfStatement" &&
          child === parent.consequent &&
          isMainGuard(parent.test)
        )
          guarded = true;
      }
      if ((guarded && options.allowInImportMetaMain !== false) || suppressed(node)) return;
      context.report({
        node,
        messageId: "topLevelAwait",
        data: {
          message:
            options.message ||
            "Top-level await suspends module evaluation and can expose uninitialized exports through barrels. Move asynchronous initialization into an exported async function and call it explicitly from an entrypoint.",
        },
      });
    }
    return {
      AwaitExpression: check,
      ForOfStatement(node) {
        if (node.await) check(node);
      },
      VariableDeclaration(node) {
        if (node.kind === "await using") check(node);
      },
    };
  },
};
