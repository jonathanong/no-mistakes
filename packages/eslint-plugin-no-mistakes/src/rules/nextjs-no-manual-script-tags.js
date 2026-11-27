"use strict";

const { literalString, rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");

const NEXT_FILE_PATTERN = /(?:^|[/\\])(?:app|pages)(?:[/\\]|$)/;

function isNextPath(filename) {
  return NEXT_FILE_PATTERN.test(filename.replace(/\\/g, "/"));
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
    const value =
      attribute.value?.type === "JSXExpressionContainer"
        ? attribute.value.expression
        : attribute.value;
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
  const value =
    attribute?.value?.type === "JSXExpressionContainer"
      ? attribute.value.expression
      : attribute?.value;
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
          },
          additionalProperties: false,
        },
      ],
      messages: {
        script: "Use next/script instead of a raw <script> tag.",
        structuredData: "{{message}}",
      },
    },
    (context) => {
      const options = context.options[0] || {};
      const patterns = allowedIdPatterns(options);
      const suppressed = ruleSuppression(context, "nextjs-no-manual-script-tags");
      let isNextFile = isNextPath(context.filename);
      function report(node, messageId, data) {
        if (suppressed(node)) return;
        context.report(data ? { node, messageId, data } : { node, messageId });
      }
      return {
        ImportDeclaration(node) {
          if (typeof node.source.value === "string" && node.source.value.startsWith("next/")) {
            isNextFile = true;
          }
        },
        JSXOpeningElement(node) {
          if (!isNextFile) return;
          if (node.name.type !== "JSXIdentifier" || node.name.name !== "script") return;
          if (isJsonLdScript(node)) {
            if (options.structuredDataComponent || options.structuredDataMessage) {
              report(node, "structuredData", {
                message:
                  options.structuredDataMessage ||
                  `Use <${options.structuredDataComponent}> instead of a raw JSON-LD <script> tag.`,
              });
            }
            return;
          }
          if (isAllowedInlineScript(node, options, patterns)) return;
          report(node, "script");
        },
      };
    },
  ),
  { __test: { attributeValue } },
);
