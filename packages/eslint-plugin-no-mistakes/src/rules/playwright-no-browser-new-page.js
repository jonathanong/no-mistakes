"use strict";

const { options, rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { createBrowserBindings } = require("./playwright-browser-bindings");
const { createBrowserInspector } = require("./playwright-browser-inspect");

const RULE_ID = "playwright-no-browser-new-page";

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow browser.newPage() in favor of a configured shared page fixture",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          fixtureName: { type: "string", pattern: "\\S" },
          browserFixtureName: { type: "string", pattern: "\\S" },
          registrationPackages: {
            type: "array",
            items: { type: "string", pattern: "\\S" },
            uniqueItems: true,
          },
          unanalyzable: { enum: ["finding", "ignore"] },
          message: { type: "string", pattern: "\\S" },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      noBrowserNewPage: "{{message}}",
      unanalyzable:
        "Cannot prove {{reason}} avoids {{browserFixtureName}}.newPage(). Use the shared `{{fixtureName}}` fixture directly, or make the browser usage statically visible.",
    },
  },
  (context) => {
    const option = options(context);
    const fixtureName = option.fixtureName ?? "page";
    const browserFixtureName = option.browserFixtureName ?? "browser";
    const reportUnanalyzableEnabled = (option.unanalyzable ?? "finding") === "finding";
    const defaultMessage = `Use the shared \`${fixtureName}\` fixture instead of ${browserFixtureName}.newPage(). Calling ${browserFixtureName}.newPage() creates an unmanaged context and page that bypass shared fixture setup and cleanup.`;
    const message = option.message ?? defaultMessage;
    const suppressed = ruleSuppression(context, RULE_ID);
    const bindings = createBrowserBindings(
      context,
      new Set(["@playwright/test", ...(option.registrationPackages ?? [])]),
    );
    const calls = [];
    const reported = new Set();

    function reportNewPage(node) {
      if (reported.has(node) || suppressed(node)) return;
      reported.add(node);
      context.report({ node, messageId: "noBrowserNewPage", data: { message } });
    }

    function reportUnanalyzable(node, reason) {
      if (!reportUnanalyzableEnabled || reported.has(node) || suppressed(node)) return;
      reported.add(node);
      context.report({
        node,
        messageId: "unanalyzable",
        data: { reason, browserFixtureName, fixtureName },
      });
    }

    const inspectCallback = createBrowserInspector(
      context,
      bindings,
      browserFixtureName,
      reportNewPage,
      reportUnanalyzable,
    );

    return {
      CallExpression(node) {
        calls.push(node);
      },
      "Program:exit"() {
        for (const call of calls) {
          const target = bindings.extractCallTarget(call);
          if (!target) continue;
          if (target.kind === "fn") inspectCallback(target.fn);
          else reportUnanalyzable(target.node, target.reason);
        }
      },
    };
  },
);
