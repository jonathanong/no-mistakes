"use strict";

const {
  attributeName,
  INTERACTIVE_ELEMENTS,
  isSelectorAttribute,
  options,
  rule,
  selectorAttributes,
  selectorLiteral,
} = require("../helpers");

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
    return {
      JSXOpeningElement(node) {
        const elementName = jsxElementName(node.name);
        const hasSelector = node.attributes.some((attr) =>
          isSelectorAttribute(attributeName(attr), attrs),
        );
        if (
          hasSelector ||
          !isInteractiveElement(elementName, node.attributes, interactiveComponents)
        ) {
          return;
        }
        context.report({ node: node.name, messageId: "missing" });
      },
    };
  },
);

function isInteractiveElement(elementName, attributes, interactiveComponents) {
  if (matchesAny(elementName, interactiveComponents)) {
    return true;
  }
  if (INTERACTIVE_ELEMENTS.has(elementName)) {
    return true;
  }
  if (elementName === "a" && attributes.some((attr) => attributeName(attr) === "href")) {
    return true;
  }
  return attributes.some((attr) => {
    if (attributeName(attr) === "onClick") {
      return true;
    }
    if (attributeName(attr) !== "role") {
      return false;
    }
    return [
      "button",
      "checkbox",
      "link",
      "menuitem",
      "option",
      "radio",
      "switch",
      "tab",
      "textbox",
    ].includes(selectorLiteral(attr));
  });
}

function jsxElementName(name) {
  if (name.type === "JSXIdentifier") {
    return name.name;
  }
  if (name.type === "JSXMemberExpression") {
    const object = jsxElementName(name.object);
    const property = jsxElementName(name.property);
    return object && property ? `${object}.${property}` : null;
  }
}

function compileMatchers(values) {
  if (!Array.isArray(values)) {
    return [];
  }
  return values
    .filter((value) => typeof value === "string" && value.length > 0)
    .map((value) => {
      const regex = regexLiteral(value);
      return regex ? { regex } : { exact: value };
    });
}

function regexLiteral(value) {
  if (!value.startsWith("/") || value.lastIndexOf("/") === 0) {
    return null;
  }
  const lastSlash = value.lastIndexOf("/");
  try {
    return new RegExp(value.slice(1, lastSlash), value.slice(lastSlash + 1));
  } catch {
    return null;
  }
}

function matchesAny(name, matchers) {
  return Boolean(
    name &&
    matchers.some((matcher) => (matcher.regex ? matcher.regex.test(name) : matcher.exact === name)),
  );
}
