"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const {
  invokedCall,
  memberName,
  methodFromCallee,
  patternKey,
  variableFor,
} = require("./playwright-static-call");

const DEFAULT_MAX_TIMEOUT_MS = 10000;
const TIMEOUT_MATCHERS = new Set([
  "poll",
  "toBeAttached",
  "toBeChecked",
  "toBeDisabled",
  "toBeEditable",
  "toBeEmpty",
  "toBeEnabled",
  "toBeFocused",
  "toBeHidden",
  "toBeInViewport",
  "toBeOK",
  "toBeVisible",
  "toContainText",
  "toHaveAttribute",
  "toHaveClass",
  "toHaveCount",
  "toHaveCSS",
  "toHaveId",
  "toHaveJSProperty",
  "toHaveRole",
  "toHaveScreenshot",
  "toHaveText",
  "toHaveTitle",
  "toHaveURL",
  "toHaveValue",
  "toHaveValues",
]);
const PLAYWRIGHT_PATH_PATTERN =
  /(?:^|[/\\])(?:e2e|playwright)(?:[/\\]|$)|(?:^|[/\\])e2e\.(?:spec|test)\.[cm]?[jt]sx?$|\.pw\.(?:spec|test)\.[cm]?[jt]sx?$/;

function isPlaywrightPath(filename) {
  return PLAYWRIGHT_PATH_PATTERN.test(filename.replace(/\\/g, "/"));
}

function isPlaywrightNamespace(node, context, seen = new Set()) {
  while (node?.type === "ChainExpression") node = node.expression;
  if (node?.type !== "Identifier") return false;
  const variable = variableFor(node, context);
  if (!variable || seen.has(variable) || variable.defs?.length !== 1) return false;
  const def = variable.defs[0];
  if (
    def.type === "ImportBinding" &&
    def.node?.type === "ImportNamespaceSpecifier" &&
    def.parent?.source?.value === "@playwright/test"
  ) {
    return true;
  }
  if (def.type !== "Variable" || def.parent?.kind !== "const" || !def.node?.init) return false;
  seen.add(variable);
  return isPlaywrightNamespace(def.node.init, context, seen);
}

function expectRoot(node, context, expectNames, seen = new Set()) {
  while (node?.type === "ChainExpression") node = node.expression;
  if (node.type === "CallExpression") return expectRoot(node.callee, context, expectNames, seen);
  if (node.type === "MemberExpression") {
    if (memberName(node) === "expect") return isPlaywrightNamespace(node.object, context);
    return expectRoot(node.object, context, expectNames, seen);
  }
  if (node.type !== "Identifier") return false;
  if (expectNames.has(node.name) || node.name === "expect") {
    const variable = variableFor(node, context);
    if (!variable || variable.defs.length === 0) return true;
    if (variable.defs.length !== 1) return false;
    const def = variable.defs[0];
    if (def.type === "ImportBinding") return true;
    return def.type === "Variable" && def.parent?.kind === "const";
  }
  if (seen.has(node.name)) return false;
  seen.add(node.name);
  const variable = variableFor(node, context);
  const def = variable?.defs?.length === 1 ? variable.defs[0] : null;
  if (def?.type === "Variable" && def.parent?.kind === "const" && def.node?.init) {
    if (def.node.id?.type === "ObjectPattern") return isPlaywrightNamespace(def.node.init, context);
    return expectRoot(def.node.init, context, expectNames, seen);
  }
  return methodFromCallee(node, context, seen) === "expect";
}

function numericLiteral(node) {
  return node?.type === "Literal" && typeof node.value === "number" ? node.value : null;
}

function propertyName(node) {
  if (typeof node.name === "string" && node.type !== "Literal") return node.name;
  return patternKey({ type: "Property", computed: true, key: node });
}

module.exports = Object.assign(
  rule(
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
      const suppressed = ruleSuppression(context, "playwright-assertion-timeout-cap");
      const max = context.options[0]?.max ?? DEFAULT_MAX_TIMEOUT_MS;
      let isPlaywrightFile = isPlaywrightPath(context.filename);
      const expectNames = new Set(["expect"]);
      return {
        ImportDeclaration(node) {
          if (node.source.value !== "@playwright/test") return;
          isPlaywrightFile = true;
          for (const specifier of node.specifiers) {
            if (specifier.type !== "ImportSpecifier") continue;
            const imported =
              specifier.imported?.type === "Identifier" ? specifier.imported.name : null;
            if (imported === "expect" && specifier.local?.name)
              expectNames.add(specifier.local.name);
          }
        },
        CallExpression(node) {
          if (!isPlaywrightFile || suppressed(node)) return;
          const invoked = invokedCall(node);
          if (!invoked || invoked.callee?.type !== "MemberExpression") return;
          const method = memberName(invoked.callee);
          if (!TIMEOUT_MATCHERS.has(method)) return;
          if (!expectRoot(invoked.callee.object, context, expectNames)) return;
          const options = invoked.arguments.at(-1);
          if (options?.type !== "ObjectExpression") return;
          const timeout = options.properties.find(
            (property) => property.type === "Property" && patternKey(property) === "timeout",
          );
          if (numericLiteral(timeout?.value) > max) {
            context.report({ node, messageId: "timeout", data: { max } });
          }
        },
      };
    },
  ),
  { __test: { propertyName } },
);
