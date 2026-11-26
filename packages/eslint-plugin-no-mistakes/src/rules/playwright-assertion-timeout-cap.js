"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression } = require("./async-ast");
const { importSpecifierName } = require("./async-target-sources");
const {
  admitsPlaywrightModule,
  reportUnanalyzable,
  resolveModuleBinding,
} = require("./module-binding");
const { invokedCall, memberName, patternKey, variableFor } = require("./playwright-static-call");

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

function isUnreadSource(source) {
  return (
    typeof source === "string" &&
    (source.startsWith(".") || source.startsWith("/") || source.startsWith("file:"))
  );
}

function expectFact(fact) {
  if (!fact) return null;
  if (fact.status === "opaque") return "opaque";
  if (fact.source === "@playwright/test") {
    if (fact.status === "namespace") return false;
    return fact.exported === "expect";
  }
  if (!isUnreadSource(fact.source)) return false;
  // import { expect } from "./fixtures" kept the cap check by local name.
  if (fact.status === "export" && fact.exported === "expect") return null;
  return "opaque";
}

function expectRoot(node, context, expectNames, seen = new Set()) {
  node = unwrapExpression(node);
  if (!node) return false;
  if (node.type === "CallExpression") return expectRoot(node.callee, context, expectNames, seen);
  if (node.type === "MemberExpression") {
    const fact = expectFact(resolveModuleBinding(node, context, seen));
    if (fact !== null && fact !== false) return fact;
    if (memberName(node) === "expect") return namespaceExpect(node.object, context, seen);
    return expectRoot(node.object, context, expectNames, seen);
  }
  if (node.type !== "Identifier") return false;
  const fact = expectFact(resolveModuleBinding(node, context, seen));
  if (fact !== null && fact !== false) return fact;
  if (fact === false) return false;
  const variable = variableFor(node, context);
  const named = expectNames.has(node.name) || node.name === "expect";
  if (!named) return false;
  if (!variable || variable.defs.length === 0) return true;
  return variable.defs[0]?.type === "ImportBinding";
}

function namespaceExpect(node, context, seen) {
  const fact = resolveModuleBinding(node, context, seen);
  if (fact?.status === "opaque" || (fact && isUnreadSource(fact.source))) return "opaque";
  return fact?.status === "namespace" && fact.source === "@playwright/test";
}

function numericLiteral(node) {
  const target = unwrapExpression(node);
  return target?.type === "Literal" && typeof target.value === "number" ? target.value : null;
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
          properties: {
            max: { type: "number" },
            unanalyzable: { enum: ["finding", "ignore"] },
          },
          additionalProperties: false,
        },
      ],
      messages: {
        timeout:
          "Assertion timeout must not exceed {{max}} ms. Increase the test timeout or fix the slow condition instead.",
        unanalyzable:
          "Cannot tell whether this assertion is Playwright expect. Import or require @playwright/test with a static name so the timeout cap can be checked.",
      },
    },
    (context) => {
      const suppressed = ruleSuppression(context, "playwright-assertion-timeout-cap");
      const option = context.options[0] ?? {};
      const max = option.max ?? DEFAULT_MAX_TIMEOUT_MS;
      const surfaceOpaque = reportUnanalyzable(option);
      let isPlaywrightFile = isPlaywrightPath(context.filename);
      const expectNames = new Set(["expect"]);
      function admit(node) {
        if (admitsPlaywrightModule(node, context)) isPlaywrightFile = true;
      }
      return {
        ImportDeclaration(node) {
          admit(node);
          if (node.source.value !== "@playwright/test") return;
          for (const specifier of node.specifiers) {
            if (specifier.type !== "ImportSpecifier" || !specifier.local?.name) continue;
            if (importSpecifierName(specifier) === "expect") expectNames.add(specifier.local.name);
          }
        },
        TSImportEqualsDeclaration: admit,
        VariableDeclarator: admit,
        ExpressionStatement: admit,
        CallExpression(node) {
          if (!isPlaywrightFile || suppressed(node)) return;
          const invoked = invokedCall(node);
          if (!invoked || invoked.callee?.type !== "MemberExpression") return;
          const method = memberName(invoked.callee);
          if (!TIMEOUT_MATCHERS.has(method)) return;
          const root = expectRoot(invoked.callee.object, context, expectNames);
          const options = unwrapExpression(invoked.arguments.at(-1));
          const timeout =
            options?.type === "ObjectExpression"
              ? options.properties.find(
                  (property) => property.type === "Property" && patternKey(property) === "timeout",
                )
              : undefined;
          if (root === "opaque") {
            if (surfaceOpaque && timeout) context.report({ node, messageId: "unanalyzable" });
            return;
          }
          if (!root || !timeout) return;
          if (numericLiteral(timeout.value) > max) {
            context.report({ node, messageId: "timeout", data: { max } });
          }
        },
      };
    },
  ),
  { __test: { propertyName } },
);
