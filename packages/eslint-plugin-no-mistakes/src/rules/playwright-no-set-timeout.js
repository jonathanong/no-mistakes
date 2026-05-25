"use strict";

const { rule } = require("../helpers");

const PLAYWRIGHT_PATH_PATTERN =
  /(?:^|[/\\])(?:e2e|playwright)(?:[/\\]|$)|(?:^|[/\\])e2e\.(?:spec|test)\.[cm]?[jt]sx?$|\.pw\.(?:spec|test)\.[cm]?[jt]sx?$/;

function isPlaywrightPath(filename) {
  return PLAYWRIGHT_PATH_PATTERN.test(filename.replace(/\\/g, "/"));
}

function isWaitForTimeout(node) {
  return (
    node.callee.type === "MemberExpression" &&
    propertyName(node.callee.property, node.callee.computed) === "waitForTimeout"
  );
}

function propertyName(node, computed = false) {
  if (node.type === "Literal") return String(node.value);
  if (computed) return null;
  return node.name;
}

function isGlobalSetTimeout(node, context) {
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const get = scope.set?.get;
    const variable =
      typeof get === "function"
        ? get.call(scope.set, "setTimeout")
        : scope.variables.find((candidate) => candidate.name === "setTimeout");
    if (variable) return variable.defs.length === 0;
    scope = scope.upper;
  }
  return true;
}

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow fixed sleeps in Playwright tests", recommended: false },
    schema: [],
    messages: {
      timeout:
        "Do not use fixed sleeps (setTimeout()/waitForTimeout()) in Playwright tests. Wait for an observable condition instead.",
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
        if (
          (node.callee.type === "Identifier" &&
            node.callee.name === "setTimeout" &&
            isGlobalSetTimeout(node.callee, context)) ||
          isWaitForTimeout(node)
        ) {
          context.report({ node, messageId: "timeout" });
        }
      },
    };
  },
);
