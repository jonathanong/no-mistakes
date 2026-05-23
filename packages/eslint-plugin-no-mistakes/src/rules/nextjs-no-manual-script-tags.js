"use strict";

const { literalString, rule } = require("../helpers");

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

module.exports = rule(
  {
    type: "problem",
    docs: { description: "prefer next/script over raw script JSX tags", recommended: false },
    schema: [
      {
        type: "object",
        properties: {
          allowInlineScriptIds: { type: "array", items: { type: "string" } },
          allowInlineScriptIdPatterns: { type: "array", items: { type: "string" } },
        },
        additionalProperties: false,
      },
    ],
    messages: { script: "Use next/script instead of a raw <script> tag." },
  },
  (context) => {
    const options = context.options[0] || {};
    const patterns = allowedIdPatterns(options);
    let isNextFile = isNextPath(context.filename);
    return {
      ImportDeclaration(node) {
        if (typeof node.source.value === "string" && node.source.value.startsWith("next/")) {
          isNextFile = true;
        }
      },
      JSXOpeningElement(node) {
        if (!isNextFile) return;
        if (node.name.type !== "JSXIdentifier" || node.name.name !== "script") return;
        if (isJsonLdScript(node)) return;
        if (isAllowedInlineScript(node, options, patterns)) return;
        context.report({ node, messageId: "script" });
      },
    };
  },
);
