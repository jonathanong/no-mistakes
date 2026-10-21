"use strict";
const { createTargetMatcher } = require("./async-targets");
const { unwrapExpression } = require("./async-ast");
const { logicalCall } = require("./module-mock-helpers");
const { resolveVariable, memberPropertyName } = require("./async-target-bindings");
const { ruleSuppression } = require("../rule-suppression");
const { inspectArgv, token } = require("./gh-api-argv");

function wrapperName(node, context, stable) {
  node = unwrapExpression(node);
  const names = [];
  while (node?.type === "MemberExpression") {
    const name = memberPropertyName(node);
    if (!name) return null;
    names.unshift(name);
    node = unwrapExpression(node.object);
  }
  if (node?.type !== "Identifier") return null;
  const variable = resolveVariable(node, context);
  // A configured module wrapper never means a same-spelled nested parameter.
  if (variable && !["module", "global"].includes(variable.scope.type)) return null;
  if (variable) {
    if (!stable.has(variable))
      stable.set(
        variable,
        !variable.references.some((reference) => reference.isWrite() && !reference.init),
      );
    if (!stable.get(variable)) return null;
  }
  return [node.name, ...names].join(".");
}
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
    const stable = new WeakMap();
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
        const invoked = logicalCall(node);
        if (invoked && matcher.resolveCallTarget(node) && token(invoked.arguments[0]) === "gh") {
          array = unwrapExpression(invoked.arguments[1]);
        } else {
          const name = wrapperName(node.callee, context, stable);
          const wrapper = (options.wrappers || []).find((entry) => entry.callee === name);
          if (!wrapper) return;
          array = unwrapExpression(node.arguments[wrapper.argumentPosition - 1]);
          includesApi = wrapper.includesApi !== false;
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
