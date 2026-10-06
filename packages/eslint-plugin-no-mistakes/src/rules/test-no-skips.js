"use strict";
const { rule, options } = require("../helpers");
const { memberName, testBinding, isContextSkip } = require("./test-no-skips-bindings");
const { ruleSuppression } = require("../rule-suppression");
const MODIFIERS = ["skip", "skipIf", "runIf", "only", "todo", "fixme"];
module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "reject skipped, conditional, pending, and focused tests",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: { allow: { type: "array", items: { enum: MODIFIERS }, uniqueItems: true } },
        additionalProperties: false,
      },
    ],
    messages: {
      modifier:
        "Test modifier '{{modifier}}' can remove CI coverage. Run the complete test and provide its required environment, or explicitly allow this modifier.",
    },
  },
  (context) => {
    const suppressed = ruleSuppression(context, "test-no-skips");
    const banned = new Set(
      MODIFIERS.filter((name) => !(options(context).allow || []).includes(name)),
    );
    return {
      MemberExpression(node) {
        const modifier = memberName(node);
        if (!banned.has(modifier) || suppressed(node.property)) return;
        const test = testBinding(node.object, context);
        const contextCall =
          modifier === "skip" &&
          node.parent.type === "CallExpression" &&
          node.parent.callee === node &&
          isContextSkip(node, context);
        if (!test && !contextCall) return;
        context.report({ node: node.property, messageId: "modifier", data: { modifier } });
      },
    };
  },
);
