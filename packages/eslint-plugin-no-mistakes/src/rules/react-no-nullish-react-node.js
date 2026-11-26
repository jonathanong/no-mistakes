"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { reportsUnanalyzable } = require("./import-resolution");
const { createReactNodeFacts } = require("../react-node-types");
const { createNullishState } = require("./react-nullish-scope");

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow nullish coalescing on ReactNode-like values",
      recommended: true,
    },
    schema: [
      {
        type: "object",
        properties: { unanalyzable: { enum: ["finding", "ignore"] } },
        additionalProperties: false,
      },
    ],
    messages: {
      nullish:
        "Do not use ?? with ReactNode values. React renders null, false, and empty values differently; use an explicit undefined check for fallbacks.",
      unanalyzable:
        "Could not resolve whether this value is a ReactNode. Use a static ReactNode import.",
    },
  },
  (context) => {
    const suppressed = ruleSuppression(context, "react-no-nullish-react-node");
    const state = createNullishState(context);
    return {
      Program(node) {
        state.setFacts(createReactNodeFacts(node));
        state.pushScope("program");
      },
      "Program:exit": state.popScope,
      FunctionDeclaration: state.enterFunction,
      "FunctionDeclaration:exit": state.popScope,
      FunctionExpression: state.enterFunction,
      "FunctionExpression:exit": state.popScope,
      ArrowFunctionExpression: state.enterFunction,
      "ArrowFunctionExpression:exit": state.popScope,
      BlockStatement() {
        state.pushScope("block");
      },
      "BlockStatement:exit": state.popScope,
      VariableDeclarator: state.defineVariable,
      LogicalExpression(node) {
        if (node.operator !== "??" || suppressed(node)) return;
        const status = state.expressionStatus(node.left);
        if (status === "yes") context.report({ node, messageId: "nullish" });
        else if (status === "unresolved" && reportsUnanalyzable(context)) {
          context.report({ node, messageId: "unanalyzable" });
        }
      },
    };
  },
);
