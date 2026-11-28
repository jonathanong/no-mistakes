"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { reportsUnanalyzable } = require("./import-resolution");
const { staticName } = require("./react-use-callee");
const callee = require("./react-use-callee");

const {
  forwardedArgument,
  isAliasedPromiseResolve,
  isPromiseResolve,
  isReactNamespace,
  isReactUse,
  ordinaryUseCall,
  patternKey,
  reactUseStatus,
} = callee;

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: { description: "disallow React.use(Promise.resolve(...))", recommended: false },
      schema: [
        {
          type: "object",
          properties: { unanalyzable: { enum: ["finding", "ignore"] } },
          additionalProperties: false,
        },
      ],
      messages: {
        resolve: "Avoid React.use(Promise.resolve(...)); pass the promise directly or use await.",
        unanalyzable:
          'Could not resolve whether this call is React.use. Use a static import from "react".',
      },
    },
    (context) => {
      const suppressed = ruleSuppression(context, "react-no-use-promise-resolve");
      const useNames = new Set();
      return {
        ImportDeclaration(node) {
          if (node.source.value !== "react" || node.importKind === "type") return;
          for (const specifier of node.specifiers) {
            if (specifier.type !== "ImportSpecifier" || specifier.importKind === "type") continue;
            if (staticName(specifier.imported, false) === "use") useNames.add(specifier.local.name);
          }
        },
        CallExpression(node) {
          if (suppressed(node)) return;
          const status = reactUseStatus(node.callee, context, useNames);
          if (status === "no") return;
          const argument = forwardedArgument(node, context);
          const resolved =
            isPromiseResolve(argument) ||
            (ordinaryUseCall(node, context) && isAliasedPromiseResolve(node.arguments[0], context));
          if (status === "yes" && resolved) {
            context.report({ node, messageId: "resolve" });
            return;
          }
          if (status === "unresolved" && resolved && reportsUnanalyzable(context)) {
            context.report({ node, messageId: "unanalyzable" });
          }
        },
      };
    },
  ),
  {
    __test: { isPromiseResolve, isReactNamespace, isReactUse, patternKey, staticName },
  },
);
