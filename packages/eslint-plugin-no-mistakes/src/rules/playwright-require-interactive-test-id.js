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
  matchesAny,
  rememberAlias,
  rememberImport,
} = require("./playwright-interactive-names");
const {
  childKeepsSelector,
  hasSlotProp,
  literalSelectors,
  singleElementChild,
} = require("./playwright-interactive-slots");

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
          slotProps: { type: "array", items: { type: "string" } },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      missing: "Interactive elements should have a test ID.",
      slot: "Use a literal test ID on this slot parent, after any spreads, so its child inherits a stable selector.",
    },
  },
  (context) => {
    const opts = options(context);
    const attrs = selectorAttributes(opts);
    const interactiveComponents = compileMatchers(opts.interactiveComponents);
    const slotProps = opts.slotProps ?? ["asChild"];
    const importedNames = new Map();
    const openings = new WeakMap();
    const suppressed = ruleSuppression(context, "playwright-require-interactive-test-id");
    function elementNames(node) {
      return jsxCandidateNames(node.name).flatMap((name) => {
        const imported = mappedImport(name, node, context, importedNames);
        return imported && imported !== name ? [name, imported] : [name];
      });
    }
    function openingInfo(node) {
      if (!openings.has(node)) {
        const names = elementNames(node);
        // Intrinsic JSX tags cannot merge props onto a child, even if configured.
        const component =
          node.name.type === "JSXMemberExpression" ||
          (node.name.type === "JSXIdentifier" && !/^[a-z]|-/.test(node.name.name));
        const enabled =
          component &&
          hasSlotProp(node, slotProps) &&
          names.some((name) => matchesAny(name, interactiveComponents));
        // Reuse the parent's child/prop facts instead of rescanning wide sibling lists.
        openings.set(node, {
          names,
          slot: enabled
            ? {
                selectors: literalSelectors(node, attrs),
                child: singleElementChild(node.parent),
              }
            : null,
        });
      }
      return openings.get(node);
    }
    function coveredByParent(node) {
      const element = node.parent;
      const parent = element.parent;
      if (parent.type !== "JSXElement") return false;
      const { slot } = openingInfo(parent.openingElement);
      if (!slot || slot.child !== element) return false;
      // A missing/invalid parent ID has one actionable finding, on that parent.
      return slot.selectors.length === 0 || childKeepsSelector(node, slot.selectors);
    }
    return {
      ImportDeclaration(node) {
        for (const specifier of node.specifiers) rememberImport(specifier, importedNames);
      },
      VariableDeclarator(node) {
        rememberAlias(node, importedNames, context);
      },
      JSXOpeningElement(node) {
        const { names, slot } = openingInfo(node);
        if (slot) {
          if (slot.selectors.length === 0 && !suppressed(node))
            context.report({ node: node.name, messageId: "slot" });
          return;
        }
        const hasSelector = node.attributes.some((attr) =>
          isSelectorAttribute(attributeName(attr), attrs),
        );
        if (
          hasSelector ||
          !isInteractiveElement(names, node.attributes, interactiveComponents) ||
          coveredByParent(node)
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
