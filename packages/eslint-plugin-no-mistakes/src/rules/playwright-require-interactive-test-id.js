"use strict";

const { ruleSuppression } = require("../rule-suppression");
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
    const importedNames = new Map();
    const suppressed = ruleSuppression(context, "playwright-require-interactive-test-id");
    return {
      ImportDeclaration(node) {
        for (const specifier of node.specifiers) {
          if (specifier.type !== "ImportSpecifier" || !specifier.local?.name) continue;
          const imported =
            specifier.imported?.type === "Identifier"
              ? specifier.imported.name
              : specifier.imported?.value;
          if (typeof imported === "string") importedNames.set(specifier.local.name, imported);
        }
      },
      VariableDeclarator(node) {
        if (node.parent?.kind !== "const") return;
        if (node.id?.type !== "Identifier" || node.init?.type !== "Identifier") return;
        const imported = importedNames.get(node.init.name);
        if (imported && !importedNames.has(node.id.name)) importedNames.set(node.id.name, imported);
      },
      JSXOpeningElement(node) {
        const elementNames = jsxCandidateNames(node.name).flatMap((name) => {
          const imported = mappedImport(name, node, context, importedNames);
          return imported && imported !== name ? [name, imported] : [name];
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
        if (suppressed(node)) return;
        context.report({ node: node.name, messageId: "missing" });
      },
    };
  },
);

function variableNamed(name, node, context) {
  if (!context?.sourceCode?.getScope) return null;
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.set?.get(name) ?? scope.variables?.find((item) => item.name === name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

// The file-wide rename map applies only to the import or a single const alias
// of that import. A nested component that reuses the local name is not that import.
function mappedImport(name, node, context, importedNames) {
  const imported = importedNames.get(name);
  if (!imported || imported === name) return null;
  const variable = variableNamed(name, node, context);
  if (!variable || variable.defs?.length !== 1) return null;
  const def = variable.defs[0];
  if (def.type === "ImportBinding") return imported;
  if (
    def.type === "Variable" &&
    def.parent?.kind === "const" &&
    def.node?.init?.type === "Identifier" &&
    importedNames.get(def.node.init.name) === imported
  ) {
    return imported;
  }
  return null;
}

function isInteractiveElement(elementNames, attributes, interactiveComponents) {
  const names = Array.isArray(elementNames) ? elementNames : [elementNames];
  if (names.some((name) => matchesAny(name, interactiveComponents))) {
    return true;
  }
  if (names.some((name) => INTERACTIVE_ELEMENTS.has(name))) {
    return true;
  }
  if (names.includes("a") && attributes.some((attr) => attributeName(attr) === "href")) {
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

function jsxCandidateNames(name) {
  if (!name) return [];
  if (name.type === "JSXIdentifier") return [name.name];
  if (name.type === "JSXMemberExpression") {
    const object = jsxElementName(name.object);
    const property = jsxElementName(name.property);
    const names = [];
    if (object && property) names.push(`${object}.${property}`);
    if (property) names.push(property);
    return names;
  }
  return [];
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
    matchers.some((matcher) => {
      if (!matcher.regex) {
        return matcher.exact === name;
      }
      matcher.regex.lastIndex = 0;
      return matcher.regex.test(name);
    }),
  );
}

module.exports.__test = { compileMatchers };
