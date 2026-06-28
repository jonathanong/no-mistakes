"use strict";

const { rule } = require("../helpers");
const helpers = require("./no-global-fetch-outside-helper-helpers");

const {
  collectFetchAliases,
  isGlobalFetchExpression,
  recordObjectPatternFetchAliases,
  setAlias,
  shouldCheckFile,
} = helpers;

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: {
        description: "disallow global fetch outside configured helper paths",
        recommended: false,
      },
      schema: [
        {
          type: "object",
          properties: {
            checkedPathPatterns: { type: "array", items: { type: "string" } },
            allowedPathPatterns: { type: "array", items: { type: "string" } },
          },
          additionalProperties: false,
        },
      ],
      messages: {
        globalFetch:
          "Move global fetch calls into a configured API/client helper so request behavior stays centralized.",
      },
    },
    (context) => {
      const options = context.options?.[0] ?? {};
      if (!shouldCheckFile(context.filename, options)) return {};
      const aliases = new Set();

      return {
        Program(node) {
          collectFetchAliases(node, context, aliases);
        },
        VariableDeclarator(node) {
          if (node.parent?.type === "VariableDeclaration" && node.parent.kind === "const") return;
          if (node.id?.type === "Identifier") {
            setAlias(
              node.id,
              isGlobalFetchExpression(node.init, context, aliases),
              context,
              aliases,
            );
            return;
          }
          recordObjectPatternFetchAliases(node.id, node.init, context, aliases);
        },
        AssignmentExpression(node) {
          if (node.operator !== "=" || node.left?.type !== "Identifier") return;
          setAlias(
            node.left,
            isGlobalFetchExpression(node.right, context, aliases),
            context,
            aliases,
          );
        },
        CallExpression(node) {
          if (!isGlobalFetchExpression(node.callee, context, aliases)) return;
          context.report({ node: node.callee, messageId: "globalFetch" });
        },
      };
    },
  ),
  { __test: helpers },
);
