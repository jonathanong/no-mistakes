"use strict";

const { literalString, rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression } = require("./async-ast");
const { reportsUnanalyzable } = require("./import-resolution");
const { nextImportState } = require("./nextjs-script-imports");

const NEXT_FILE_PATTERN = /(?:^|[/\\])(?:app|pages)(?:[/\\]|$)/;

function isNextPath(filename) {
  return NEXT_FILE_PATTERN.test(String(filename || "").replace(/\\/g, "/"));
}

function attributeExpression(attribute) {
  if (attribute?.value?.type === "JSXExpressionContainer") return attribute.value.expression;
  return attribute?.value;
}

function isJsonLdScript(node) {
  return node.attributes.some((attribute) => {
    if (
      attribute.type !== "JSXAttribute" ||
      attribute.name.type !== "JSXIdentifier" ||
      attribute.name.name !== "type"
    ) {
      return false;
    }
    const value = unwrapExpression(attributeExpression(attribute));
    return value?.type === "Literal" && value.value === "application/ld+json";
  });
}

function jsxAttribute(node, name) {
  return node.attributes.find(
    (attribute) =>
      attribute.type === "JSXAttribute" &&
      attribute.name.type === "JSXIdentifier" &&
      attribute.name.name === name,
  );
}

function attributeValue(attribute) {
  const value = attributeExpression(attribute);
  return value ? literalString(value) : null;
}

function hasAttribute(node, name) {
  return Boolean(jsxAttribute(node, name));
}

function allowedIdPatterns(options) {
  return (options.allowInlineScriptIdPatterns || []).flatMap((pattern) => {
    try {
      return [new RegExp(pattern)];
    } catch {
      return [];
    }
  });
}

function isAllowedInlineScript(node, options, patterns) {
  if (!hasAttribute(node, "dangerouslySetInnerHTML")) return false;
  const id = attributeValue(jsxAttribute(node, "id"));
  if (!id) return false;
  return (
    (options.allowInlineScriptIds || []).includes(id) || patterns.some((regex) => regex.test(id))
  );
}

function isScriptTag(node) {
  return node.name.type === "JSXIdentifier" && node.name.name === "script";
}

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: { description: "prefer next/script over raw script JSX tags", recommended: false },
      schema: [
        {
          type: "object",
          properties: {
            allowInlineScriptIds: { type: "array", items: { type: "string" } },
            allowInlineScriptIdPatterns: { type: "array", items: { type: "string" } },
            structuredDataComponent: { type: "string", minLength: 1 },
            structuredDataMessage: { type: "string", minLength: 1 },
            unanalyzable: { enum: ["finding", "ignore"] },
          },
          additionalProperties: false,
        },
      ],
      messages: {
        script: "Use next/script instead of a raw <script> tag.",
        structuredData: "{{message}}",
        unanalyzable:
          "Could not resolve whether this file imports Next.js. Use a static next/ import.",
      },
    },
    (context) => {
      const options = context.options[0] || {};
      const patterns = allowedIdPatterns(options);
      const suppressed = ruleSuppression(context, "nextjs-no-manual-script-tags");
      let isNextFile = isNextPath(context.filename);
      let nextUnresolved = false;
      function report(node, messageId, data) {
        if (suppressed(node)) return;
        context.report(data ? { node, messageId, data } : { node, messageId });
      }
      return {
        Program(node) {
          if (isNextFile) return;
          const state = nextImportState(node, context);
          isNextFile = state.next;
          nextUnresolved = state.unresolved && !state.next;
        },
        JSXOpeningElement(node) {
          if (!isScriptTag(node)) return;
          const structured = options.structuredDataComponent || options.structuredDataMessage;
          if (isJsonLdScript(node)) {
            if (!isNextFile) {
              if (structured && nextUnresolved && reportsUnanalyzable(context)) {
                report(node, "unanalyzable");
              }
              return;
            }
            if (structured) {
              report(node, "structuredData", {
                message:
                  options.structuredDataMessage ||
                  `Use <${options.structuredDataComponent}> instead of a raw JSON-LD <script> tag.`,
              });
            }
            return;
          }
          if (isAllowedInlineScript(node, options, patterns)) return;
          if (!isNextFile) {
            if (nextUnresolved && reportsUnanalyzable(context)) report(node, "unanalyzable");
            return;
          }
          report(node, "script");
        },
      };
    },
  ),
  { __test: { attributeValue } },
);
