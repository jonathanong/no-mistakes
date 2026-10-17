"use strict";

const { ruleSuppression } = require("../rule-suppression");

const DEFAULT_MESSAGE =
  "This comment matches a configured phrase. Remove the preserved path, or suppress a genuine external constraint with a nonempty reason.";

// Unique members of the JS RegExp flag set. Repeats and other characters are
// config errors, not one invalidPattern diagnostic per linted file.
const FLAG_PATTERN = "^(?!.*(.).*\\1)[dgimsuvy]*$";

// Compiled once per file application. `g` and `y` keep state on the RegExp, so
// each comment is tested once from index 0. A zero-width success reports that
// comment once and cannot loop; sticky patterns match only at the start.
function compilePatterns(patterns) {
  return patterns.map((entry) => {
    try {
      return {
        regex: new RegExp(entry.pattern, entry.flags ?? ""),
        message: entry.message,
        pattern: entry.pattern,
      };
    } catch (error) {
      return { error, pattern: entry.pattern };
    }
  });
}

function matches(regex, value) {
  regex.lastIndex = 0;
  const matched = regex.test(value);
  regex.lastIndex = 0;
  return matched;
}

module.exports = {
  meta: {
    type: "problem",
    docs: {
      description: "flag configured phrases in line and block comments",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          patterns: {
            type: "array",
            maxItems: 64,
            items: {
              type: "object",
              properties: {
                pattern: { type: "string", minLength: 1 },
                flags: { type: "string", pattern: FLAG_PATTERN },
                message: { type: "string", minLength: 1 },
              },
              required: ["pattern"],
              additionalProperties: false,
            },
          },
          message: { type: "string", minLength: 1 },
          requireSuppressionReason: { type: "boolean" },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      phrase: "{{message}}",
      invalidPattern: "Invalid comment phrase pattern {{pattern}}: {{detail}}",
    },
  },
  create(context) {
    const options = context.options[0] || {};
    const compiled = compilePatterns(options.patterns || []);
    const suppressed = ruleSuppression(context, "no-comment-phrases", {
      requireReason: options.requireSuppressionReason === true,
    });
    const fallback = options.message || DEFAULT_MESSAGE;
    return {
      Program(node) {
        const ready = [];
        for (const entry of compiled) {
          if (entry.error) {
            // Program starts after a leading comment in oxlint and at byte 0 in
            // ESLint. Pin the config diagnostic to the first line so they agree.
            context.report({
              node,
              loc: { start: { line: 1, column: 0 }, end: { line: 1, column: 0 } },
              messageId: "invalidPattern",
              data: { pattern: entry.pattern, detail: entry.error.message },
            });
          } else ready.push(entry);
        }
        if (ready.length === 0) return;
        for (const comment of context.sourceCode.getAllComments()) {
          // Shebang comments are neither line nor block comment tokens.
          if (comment.type !== "Line" && comment.type !== "Block") continue;
          if (suppressed(comment)) continue;
          for (const entry of ready) {
            if (!matches(entry.regex, comment.value)) continue;
            context.report({
              node,
              loc: comment.loc,
              messageId: "phrase",
              data: { message: entry.message || fallback },
            });
          }
        }
      },
    };
  },
};
