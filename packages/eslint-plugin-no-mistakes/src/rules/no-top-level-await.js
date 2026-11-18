"use strict";
const { ruleSuppression } = require("../rule-suppression");

function isMainGuard(node) {
  while (
    [
      "TSAsExpression",
      "TSTypeAssertion",
      "TSNonNullExpression",
      "TSSatisfiesExpression",
      "ParenthesizedExpression",
    ].includes(node.type)
  ) {
    node = node.expression;
  }
  return (
    node.type === "MemberExpression" &&
    !node.computed &&
    node.property.name === "main" &&
    node.object.type === "MetaProperty" &&
    node.object.meta.name === "import"
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
          message: { type: "string", minLength: 1 },
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
    };
  },
};
