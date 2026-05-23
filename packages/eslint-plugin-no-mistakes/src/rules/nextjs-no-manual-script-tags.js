"use strict";

const { rule } = require("../helpers");

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

module.exports = rule(
  {
    type: "problem",
    docs: { description: "prefer next/script over raw script JSX tags", recommended: false },
    schema: [],
    messages: { script: "Use next/script instead of a raw <script> tag." },
  },
  (context) => {
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
        context.report({ node, messageId: "script" });
      },
    };
  },
);
