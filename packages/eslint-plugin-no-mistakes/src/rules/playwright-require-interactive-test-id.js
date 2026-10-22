"use strict";

const { ruleSuppression } = require("../rule-suppression");
const {
  attributeName,
  isSelectorAttribute,
  options,
  rule,
  selectorAttributes,
} = require("../helpers");
const {
  isInteractiveElement,
  jsxCandidateNames,
  mappedImport,
  rememberAlias,
  rememberImport,
} = require("./playwright-interactive-names");

module.exports = rule(
  {
    type: "suggestion",
    docs: { description: "require test IDs on interactive JSX elements", recommended: false },
    schema: [
      {
        type: "object",
        properties: {
          selectorAttributes: { type: "array", items: { type: "string" } },
          interactiveComponents: { type: "array", items: { type: "string" } },
        },
        additionalProperties: false,
      },
    ],
    messages: { missing: "Interactive elements should have a test ID." },
  },
  (context) => {
    const opts = options(context);
    const attrs = selectorAttributes(opts);
    const interactiveComponents = compileMatchers(opts.interactiveComponents);
    const importedNames = new Map();
    const suppressed = ruleSuppression(context, "playwright-require-interactive-test-id");
    return {
      ImportDeclaration(node) {
        for (const specifier of node.specifiers) rememberImport(specifier, importedNames);
      },
      VariableDeclarator(node) {
        rememberAlias(node, importedNames, context);
      },
      JSXOpeningElement(node) {
        const elementNames = jsxCandidateNames(node.name).flatMap((name) => {
          const imported = mappedImport(name, node, context, importedNames);
          if (imported && imported !== name) {
            return [name, imported];
          }
          return [name];
        });
        const hasSelector = node.attributes.some((attr) =>
          isSelectorAttribute(attributeName(attr), attrs),
        );
        if (
          hasSelector ||
          !isInteractiveElement(elementNames, node.attributes, interactiveComponents)
        ) {
          return;
        }
        const hidden = suppressed(node);
        if (hidden) {
          return;
        }
        context.report({ node: node.name, messageId: "missing" });
      },
    };
  },
);

function compileMatchers(values) {
  if (!Array.isArray(values)) return [];
  return values
    .filter((value) => typeof value === "string" && value.length > 0)
    .map((value) => {
      const regex = regexLiteral(value);
      return regex ? { regex } : { exact: value };
    });
}

function regexLiteral(value) {
  if (!value.startsWith("/") || value.lastIndexOf("/") === 0) return null;
  const lastSlash = value.lastIndexOf("/");
  try {
    return new RegExp(value.slice(1, lastSlash), value.slice(lastSlash + 1));
  } catch {
    return null;
  }
}

module.exports.__test = { compileMatchers };
