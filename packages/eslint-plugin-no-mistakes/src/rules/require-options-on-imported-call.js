"use strict";

const { rule } = require("../helpers");
const { createTargetMatcher } = require("./async-targets");
const {
  compileImportedCallTargets,
  hasRequiredOptions,
  importedCallOptionsSchema,
  matchingTargets,
  staticPropertyName,
  visiblePropertyNames,
} = require("./imported-call-options");

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: {
        description:
          "require a statically visible options object on calls resolved from configured imports",
        recommended: false,
      },
      schema: importedCallOptionsSchema,
      messages: {
        missingOptions:
          "Call '{{calleeName}}' from '{{source}}' must pass a statically visible options object at argument {{optionsPosition}} containing {{propertyMatch}} of: {{requiredProperties}}.",
      },
    },
    (context) => {
      const matcher = createTargetMatcher(context);
      const optionTargets = compileImportedCallTargets(context.options?.[0] || {});
      if (!matcher.hasTargets || optionTargets.length === 0) return {};

      return {
        ...matcher.visitors,
        CallExpression(node) {
          const resolved = matcher.resolveCallTarget(node);
          if (!resolved) return;
          for (const target of matchingTargets(
            optionTargets,
            resolved.source,
            resolved.calleeName,
          )) {
            if (hasRequiredOptions(node.arguments[target.optionsPosition - 1], target)) continue;
            context.report({
              node,
              messageId: "missingOptions",
              data: {
                source: resolved.source,
                calleeName: resolved.calleeName,
                optionsPosition: String(target.optionsPosition),
                propertyMatch: target.propertyMatch,
                requiredProperties: target.requiredProperties.join(", "),
              },
            });
          }
        },
      };
    },
  ),
  {
    __test: {
      compileImportedCallTargets,
      hasRequiredOptions,
      staticPropertyName,
      visiblePropertyNames,
    },
  },
);
