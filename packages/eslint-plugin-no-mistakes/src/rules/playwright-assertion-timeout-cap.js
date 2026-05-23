"use strict";

const { rule } = require("../helpers");

const DEFAULT_MAX_TIMEOUT_MS = 10000;
const PLAYWRIGHT_PATH_PATTERN =
  /(?:^|[/\\])(?:e2e|playwright)(?:[/\\]|$)|(?:^|[/\\])e2e\.(?:spec|test)\.[cm]?[jt]sx?$|\.pw\.(?:spec|test)\.[cm]?[jt]sx?$/;

function isPlaywrightPath(filename) {
  return PLAYWRIGHT_PATH_PATTERN.test(filename.replace(/\\/g, "/"));
}

function isExpectChain(node) {
  let current = node;
  while (current) {
    if (current.type === "Identifier") return current.name === "expect";
    current = current.type === "CallExpression" ? current.callee : current.object;
  }
  return false;
}

function numericLiteral(node) {
  return node?.type === "Literal" && typeof node.value === "number" ? node.value : null;
}

function propertyName(node) {
  if (!node) return null;
  if (node.type === "Identifier") return node.name;
  return node.type === "Literal" ? String(node.value) : null;
}

module.exports = rule(
  {
    type: "problem",
    docs: { description: "cap Playwright assertion timeouts", recommended: false },
    schema: [
      {
        type: "object",
        properties: { max: { type: "number" } },
        additionalProperties: false,
      },
    ],
    messages: {
      timeout:
        "Assertion timeout must not exceed {{max}} ms. Increase the test timeout or fix the slow condition instead.",
    },
  },
  (context) => {
    const max = context.options[0]?.max ?? DEFAULT_MAX_TIMEOUT_MS;
    let isPlaywrightFile = isPlaywrightPath(context.filename);
    return {
      ImportDeclaration(node) {
        if (node.source.value === "@playwright/test") isPlaywrightFile = true;
      },
      CallExpression(node) {
        if (!isPlaywrightFile) return;
        if (node.callee.type !== "MemberExpression" || !isExpectChain(node.callee)) return;
        const method = propertyName(node.callee.property);
        if (method !== "poll" && !method?.startsWith("to")) return;
        const options = node.arguments.at(-1);
        if (options?.type !== "ObjectExpression") return;
        const timeout = options.properties.find(
          (property) => property.type === "Property" && propertyName(property.key) === "timeout",
        );
        if (numericLiteral(timeout?.value) > max) {
          context.report({ node, messageId: "timeout", data: { max } });
        }
      },
    };
  },
);
