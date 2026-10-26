"use strict";
const { createTargetMatcher } = require("./async-targets");
const { unwrapExpression } = require("./async-ast");
const { createWrapperResolver } = require("./gh-api-wrappers");
const { ruleSuppression } = require("../rule-suppression");
const { inspectArgv, token } = require("./gh-api-argv");

module.exports = {
  meta: {
    type: "problem",
    docs: {
      description: "require structured endpoint fields and explicit gh api methods",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          wrappers: {
            type: "array",
            maxItems: 64,
            items: {
              type: "object",
              properties: {
                callee: { type: "string", minLength: 1 },
                argumentPosition: { type: "integer", minimum: 1, maximum: 32 },
                includesApi: { type: "boolean" },
              },
              required: ["callee", "argumentPosition"],
              additionalProperties: false,
            },
          },
          endpointMessage: { type: "string", minLength: 1 },
          methodMessage: { type: "string", minLength: 1 },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      endpointQuery: "{{message}}",
      missingMethod: "{{message}}",
    },
  },
  create(context) {
    const options = context.options[0] || {};
    const resolveWrapper = createWrapperResolver(context, options.wrappers || []);
    const matcherContext = Object.create(context);
    Object.defineProperty(matcherContext, "options", {
      value: [
        {
          targets: [
            {
              sourceSpecifierPatterns: ["node:child_process", "child_process"],
              calleeNamePatterns: ["execFile", "execFileSync", "spawn", "spawnSync"],
            },
          ],
        },
      ],
    });
    const matcher = createTargetMatcher(matcherContext);
    const suppressed = ruleSuppression(context, "gh-api-argument-safety");
    return {
      ...matcher.visitors,
      CallExpression(node) {
        let array;
        let includesApi = true;
        const resolved = matcher.resolveCallTarget(node);
        if (resolved && token(resolved.arguments[0]) === "gh") {
          array = unwrapExpression(resolved.arguments[1]);
        } else {
          const wrapped = resolveWrapper(node);
          if (!wrapped) return;
          array = unwrapExpression(wrapped.arguments[wrapped.wrapper.argumentPosition - 1]);
          includesApi = wrapped.wrapper.includesApi !== false;
        }
        if (suppressed(node)) return;
        const result = inspectArgv(array, includesApi);
        if (!result) return;
        if (result.endpoint && !suppressed(result.endpoint))
          context.report({
            node: result.endpoint,
            messageId: "endpointQuery",
            data: {
              message:
                options.endpointMessage ||
                "Use structured -f/-F fields instead of an endpoint query string.",
            },
          });
        if (result.missingMethod)
          context.report({
            node,
            messageId: "missingMethod",
            data: {
              message:
                options.methodMessage ||
                "Set an explicit -X/--method when gh api uses field flags (for example -X GET).",
            },
          });
      },
    };
  },
};
