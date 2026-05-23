"use strict";

const { literalString, rule } = require("../helpers");

const HEADING_TAGS = new Set(["h1", "h2", "h3", "h4", "h5", "h6"]);
const ALLOWED_PREFIXES = [
  "script[",
  "meta[",
  "link[",
  "tbody",
  "thead",
  "tfoot",
  "tr",
  "td",
  "th",
  "body",
  "html",
  "[data-",
  ":has(",
  ":nth-",
  ":not(",
  ":first-",
  ":last-",
];
const PLAYWRIGHT_PATH_PATTERN = /(?:^|[/\\])(?:e2e|playwright)(?:[/\\]|\.|$)|\.pw\.(?:spec|test)\./;

function isPlaywrightPath(filename) {
  return PLAYWRIGHT_PATH_PATTERN.test(filename.replace(/\\/g, "/"));
}

function isAllowedSelector(selector) {
  return ALLOWED_PREFIXES.some((prefix) => selector.trim().startsWith(prefix));
}

module.exports = rule(
  {
    type: "suggestion",
    docs: {
      description: "prefer semantic Playwright locators over raw selectors",
      recommended: false,
    },
    schema: [],
    messages: {
      semantic: "Prefer a semantic locator such as getByRole(), getByLabel(), or getByText().",
      heading: "Use getByRole('heading', { level: N }) instead of a heading-tag locator.",
      text: "Use getByText() instead of a text= locator.",
    },
  },
  (context) => {
    let isPlaywrightFile = isPlaywrightPath(context.filename);
    return {
      ImportDeclaration(node) {
        if (node.source.value === "@playwright/test") isPlaywrightFile = true;
      },
      CallExpression(node) {
        if (!isPlaywrightFile) return;
        if (node.callee.type !== "MemberExpression") return;
        if (node.callee.property.type !== "Identifier" || node.callee.property.name !== "locator") {
          return;
        }
        const selector = literalString(node.arguments[0])?.trim();
        if (!selector || isAllowedSelector(selector)) return;
        if (/^\.[a-zA-Z_-]/.test(selector) || selector.startsWith("#")) {
          context.report({ node, messageId: "semantic" });
        } else if (HEADING_TAGS.has(selector.toLowerCase())) {
          context.report({ node, messageId: "heading" });
        } else if (selector === "text" || selector.startsWith("text=")) {
          context.report({ node, messageId: "text" });
        }
      },
    };
  },
);
