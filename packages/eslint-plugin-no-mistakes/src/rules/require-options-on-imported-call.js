"use strict";

const { ruleSuppression } = require("../rule-suppression");
const { rule } = require("../helpers");
const { createTargetMatcher } = require("./async-targets");
const {
  compileImportedCallTargets,
  hasRequiredOptions,
  importedCallOptionsSchema,
  isDefinitelyUndefinedValue,
  matchingTargets,
  staticPropertyName,
  visiblePropertyNames,
} = require("./imported-call-options");
const { shouldCheckOptionsFile } = require("./required-call-selector-compile");
const { createCallSelectors } = require("./required-call-selectors");

function reportMissing(context, node, target, resolved, suppressed, fallback) {
  if (suppressed(node)) return;
  if (hasRequiredOptions(node.arguments[target.optionsPosition - 1], target, context.sourceCode)) {
    return;
  }
  const custom = target.message || fallback;
  if (custom) {
    context.report({ node, messageId: "customOptions", data: { message: custom } });
    return;
  }
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

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: {
        description:
          "require a statically visible options object on configured imported, global, or receiver-shaped calls",
        recommended: false,
      },
      schema: importedCallOptionsSchema,
      messages: {
        missingOptions:
          "Call '{{calleeName}}' from '{{source}}' must pass a statically visible options object at argument {{optionsPosition}} containing {{propertyMatch}} of: {{requiredProperties}}.",
        customOptions: "{{message}}",
      },
    },
    (context) => {
      const options = context.options?.[0] || {};
      if (!shouldCheckOptionsFile(context.filename, options)) return {};
      const optionTargets = compileImportedCallTargets(options);
      const selectors = createCallSelectors(context, options);
      if (optionTargets.length === 0 && !selectors.hasTargets) return {};
      const matcher = optionTargets.length > 0 ? createTargetMatcher(context) : null;
      const suppressed = ruleSuppression(context, "require-options-on-imported-call");
      const fallback = typeof options.message === "string" ? options.message : "";

      return {
        ...(matcher ? matcher.visitors : {}),
        Program(node) {
          matcher?.visitors.Program(node);
          selectors.record(node);
        },
        CallExpression(node) {
          if (matcher) {
            const resolved = matcher.resolveCallTarget(node);
            if (resolved) {
              for (const target of matchingTargets(
                optionTargets,
                resolved.source,
                resolved.calleeName,
              )) {
                reportMissing(context, node, target, resolved, suppressed, fallback);
              }
            }
          }
          for (const match of selectors.matches(node)) {
            reportMissing(context, node, match.target, match.resolved, suppressed, fallback);
          }
        },
      };
    },
  ),
  {
    __test: {
      compileImportedCallTargets,
      hasRequiredOptions,
      isDefinitelyUndefinedValue,
      staticPropertyName,
      visiblePropertyNames,
    },
  },
);
