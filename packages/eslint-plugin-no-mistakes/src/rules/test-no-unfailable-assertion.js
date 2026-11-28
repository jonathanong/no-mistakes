"use strict";

const { options, rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const {
  excludesUndefined,
  getExpressionType,
  getTypeContext,
  isAlwaysTruthy,
  isDiscardedPlaywrightLocator,
} = require("./test-no-unfailable-assertion-types");
const {
  isSameValueAssertion,
  parseAssertion,
} = require("./test-no-unfailable-assertion-assertions");
const { throwingQueryName } = require("./test-no-unfailable-assertion-queries");
const { findDefinedThenDeref } = require("./test-no-unfailable-assertion-deref");
const { createThrowingQueryFix } = require("./test-no-unfailable-assertion-autofix");

const RULE_ID = "test-no-unfailable-assertion";
const DEFAULT_SEVERITIES = {
  "defined-by-type": "error",
  "same-value": "error",
  "truthy-by-type": "error",
  "throwing-query": "advisory",
  "discarded-locator": "advisory",
  "defined-then-deref": "advisory",
};
const SEVERITY_SCHEMA = { enum: ["error", "advisory", "off"] };

function isDefinedCheck(assertion) {
  return (
    (!assertion.negated && assertion.matcher === "toBeDefined") ||
    (assertion.negated && assertion.matcher === "toBeUndefined")
  );
}

function isTruthyCheck(assertion) {
  return (
    (!assertion.negated && assertion.matcher === "toBeTruthy") ||
    (assertion.negated && assertion.matcher === "toBeFalsy")
  );
}

function isThrowingQueryMatcher(assertion) {
  return (
    isDefinedCheck(assertion) ||
    isTruthyCheck(assertion) ||
    (assertion.negated && assertion.matcher === "toBeNull")
  );
}

module.exports = rule(
  {
    type: "problem",
    fixable: "code",
    docs: {
      description: "disallow assertions that cannot fail",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          severities: {
            type: "object",
            properties: Object.fromEntries(
              Object.keys(DEFAULT_SEVERITIES).map((code) => [code, SEVERITY_SCHEMA]),
            ),
            additionalProperties: false,
          },
          report: { enum: ["all", "error", "advisory"] },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      "defined-by-type":
        "{{prefix}}Assertion cannot fail: the checked value's type excludes undefined. Assert on a concrete value or behavior instead.",
      "same-value":
        "{{prefix}}Assertion cannot fail: it compares a literal or the same value against itself. Assert on the behavior under test instead.",
      "truthy-by-type":
        "{{prefix}}Assertion cannot fail: the checked value's type is always truthy. Assert on a concrete property or outcome instead.",
      "throwing-query":
        "{{prefix}}Testing Library '{{query}}' already throws when nothing matches. Assert on visible content, attributes, or state instead.",
      "discarded-locator":
        "{{prefix}}Playwright locator is created and discarded without asserting or acting on it. Await an assertion or action on the locator.",
      "defined-then-deref":
        "{{prefix}}Redundant defined check: a subsequent assertion already dereferences this value. Remove the extra toBeDefined() check or narrow with an explicit guard.",
    },
  },
  (context) => {
    const config = options(context);
    const severities = { ...DEFAULT_SEVERITIES, ...config.severities };
    const reportMode = config.report || "all";
    const suppressed = ruleSuppression(context, RULE_ID, { requireReason: true });
    const typeCtx = getTypeContext(context);
    const reportedAssertions = new Set();
    const jestDomCache = { fileOrSetup: null };

    function isEnabled(code) {
      const severity = severities[code];
      return severity !== "off" && (reportMode === "all" || reportMode === severity);
    }

    function report(node, messageId, extra = {}, fix = null) {
      if (suppressed(node)) return false;
      const prefix = severities[messageId] === "advisory" ? "[advisory] " : "";
      const descriptor = { node, messageId, data: { prefix, ...extra } };
      if (fix) descriptor.fix = fix;
      context.report(descriptor);
      return true;
    }

    function checkStatements(statements) {
      if (!isEnabled("defined-then-deref") || !Array.isArray(statements)) return;
      for (const node of findDefinedThenDeref(statements, context, reportedAssertions)) {
        report(node, "defined-then-deref");
      }
    }

    return {
      CallExpression(node) {
        const assertion = parseAssertion(node, context);
        if (!assertion) return;

        if (isEnabled("same-value") && isSameValueAssertion(assertion, context)) {
          reportedAssertions.add(node);
          report(node, "same-value");
          return;
        }

        if (assertion.asyncModifier === "rejects") return;

        if (isEnabled("throwing-query") && isThrowingQueryMatcher(assertion)) {
          const query = throwingQueryName(assertion.actual, context);
          if (query) {
            reportedAssertions.add(node);
            const fix = createThrowingQueryFix(
              node,
              assertion,
              query,
              context,
              typeCtx,
              jestDomCache,
            );
            report(node, "throwing-query", { query }, fix);
            return;
          }
        }

        const definedCheck = isDefinedCheck(assertion);
        const truthyCheck = isTruthyCheck(assertion);
        if ((!definedCheck && !truthyCheck) || !typeCtx || !typeCtx.strictNullChecks) return;

        const type = getExpressionType(
          assertion.actual,
          typeCtx,
          assertion.asyncModifier === "resolves",
        );
        if (!type) return;

        if (
          definedCheck &&
          isEnabled("defined-by-type") &&
          excludesUndefined(type, typeCtx.checker, typeCtx.ts)
        ) {
          reportedAssertions.add(node);
          report(node, "defined-by-type");
        } else if (
          truthyCheck &&
          isEnabled("truthy-by-type") &&
          isAlwaysTruthy(type, typeCtx.checker, typeCtx.ts)
        ) {
          reportedAssertions.add(node);
          report(node, "truthy-by-type");
        }
      },
      ExpressionStatement(node) {
        if (
          isEnabled("discarded-locator") &&
          typeCtx &&
          isDiscardedPlaywrightLocator(node, typeCtx)
        ) {
          report(node, "discarded-locator");
        }
      },
      "Program:exit"(node) {
        checkStatements(node.body);
      },
      "BlockStatement:exit"(node) {
        checkStatements(node.body);
      },
      "SwitchCase:exit"(node) {
        checkStatements(node.consequent);
      },
    };
  },
);
