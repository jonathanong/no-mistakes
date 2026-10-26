"use strict";

const { attributeName, INTERACTIVE_ELEMENTS, selectorLiteral } = require("../helpers");

function variableNamed(name, node, context) {
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.set.get(name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function importedSpecifierName(specifier) {
  if (specifier.type !== "ImportSpecifier") return null;
  return specifier.imported.name;
}

function rememberImport(specifier, importedNames) {
  const imported = importedSpecifierName(specifier);
  const local = specifier.local;
  if (!imported || !local) return;
  importedNames.set(local.name, imported);
}

function rememberAlias(node, importedNames, context) {
  const parent = node.parent;
  const id = node.id;
  const init = node.init;
  if (!parent || parent.kind !== "const") return;
  if (!id || id.type !== "Identifier" || !init || init.type !== "Identifier") return;
  const imported = importedNames.get(init.name);
  if (!imported) return;
  const defs = variableNamed(init.name, init, context)?.defs ?? [];
  if (defs.length === 1 && defs[0].type === "ImportBinding") {
    importedNames.set(id.name, imported);
  }
}

// The rename map applies to the import or one const alias of it. A nested
// component that reuses the local name is not that import.
function bindingImportName(def) {
  if (!def || def.type !== "ImportBinding") return null;
  return importedSpecifierName(def.node);
}

function mappedImport(name, node, context, importedNames) {
  const variable = variableNamed(name, node, context);
  if (!variable || variable.defs.length !== 1) return null;
  const def = variable.defs[0];
  const fromBinding = bindingImportName(def);
  if (fromBinding && fromBinding !== name) return fromBinding;
  const imported = importedNames.get(name);
  if (!imported || imported === name || def.type !== "Variable") return null;
  const init = def.node.init;
  const source =
    init && init.type === "Identifier" ? variableNamed(init.name, init, context) : null;
  const sourceDefs = source ? source.defs : [];
  const kind = def.parent.kind;
  if (
    kind === "const" &&
    sourceDefs.length === 1 &&
    sourceDefs[0].type === "ImportBinding" &&
    importedNames.get(init.name) === imported
  ) {
    return imported;
  }
  return null;
}

function jsxElementName(name) {
  if (name.type === "JSXIdentifier") return name.name;
  return `${jsxElementName(name.object)}.${jsxElementName(name.property)}`;
}

function jsxCandidateNames(name) {
  if (name.type === "JSXIdentifier") return [name.name];
  if (name.type !== "JSXMemberExpression") return [];
  const property = jsxElementName(name.property);
  return [`${jsxElementName(name.object)}.${property}`, property];
}

function isInteractiveElement(elementNames, attributes, interactiveComponents) {
  if (elementNames.some((name) => matchesAny(name, interactiveComponents))) return true;
  if (elementNames.some((name) => INTERACTIVE_ELEMENTS.has(name))) return true;
  if (elementNames.includes("a") && attributes.some((attr) => attributeName(attr) === "href")) {
    return true;
  }
  return attributes.some((attr) => {
    if (attributeName(attr) === "onClick") return true;
    if (attributeName(attr) !== "role") return false;
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

function matchesAny(name, matchers) {
  return Boolean(
    name &&
    matchers.some((matcher) => {
      if (!matcher.regex) return matcher.exact === name;
      matcher.regex.lastIndex = 0;
      return matcher.regex.test(name);
    }),
  );
}

module.exports = {
  importedSpecifierName,
  isInteractiveElement,
  jsxCandidateNames,
  mappedImport,
  matchesAny,
  rememberAlias,
  rememberImport,
};
