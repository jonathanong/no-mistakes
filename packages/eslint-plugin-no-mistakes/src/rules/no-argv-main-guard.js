"use strict";
const { createTargetMatcher } = require("./async-targets");
const { ruleSuppression } = require("../rule-suppression");
const { createClassifier, ARGV, MAIN } = require("./argv-main-values");
module.exports = {
  meta: {
    type: "problem",
    docs: {
      description: "avoid argv-based main-module comparisons that fail through symlinks",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: { message: { type: "string", minLength: 1 } },
        additionalProperties: false,
      },
    ],
    messages: { argvMain: "{{message}}" },
  },
  create(context) {
    const matcherContext = Object.create(context);
    Object.defineProperty(matcherContext, "options", {
      value: [
        {
          targets: [
            {
              sourceSpecifierPatterns: ["node:url", "url"],
              calleeNamePatterns: ["fileURLToPath", "pathToFileURL"],
            },
            { sourceSpecifierPatterns: ["node:path", "path"], calleeNamePatterns: ["resolve"] },
            { sourceSpecifierPatterns: ["node:fs", "fs"], calleeNamePatterns: ["realpathSync"] },
          ],
        },
      ],
    });
    const wrappers = createTargetMatcher(matcherContext);
    const classify = createClassifier(context, wrappers);
    const suppressed = ruleSuppression(context, "no-argv-main-guard");
    return {
      ...wrappers.visitors,
      BinaryExpression(node) {
        if (!["===", "!==", "==", "!="].includes(node.operator) || suppressed(node)) return;
        const left = classify(node.left);
        const right = classify(node.right);
        if (!((left === ARGV && right === MAIN) || (left === MAIN && right === ARGV))) return;
        context.report({
          node,
          messageId: "argvMain",
          data: {
            message:
              context.options[0]?.message ||
              "Use import.meta.main or inject isMain instead of comparing argv[1] with the module path; symlinked entry paths can differ.",
          },
        });
      },
    };
  },
};
