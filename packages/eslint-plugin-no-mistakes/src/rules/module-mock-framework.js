"use strict";

const FRAMEWORK_MODULES = new Set(["vitest", "@jest/globals"]);

function propertyName(node) {
  if (!node) return null;
  return node.type === "Literal" ? String(node.value) : node.name;
}

function memberPropertyName(node) {
  if (!node?.computed) return propertyName(node?.property);
  return node.property?.type === "Literal" ? String(node.property.value) : null;
}

function frameworkRequireModule(def) {
  const init = def.node?.init;
  const requireCall =
    init?.type === "MemberExpression" && memberPropertyName(init) ? init.object : init;
  if (
    def.type === "Variable" &&
    requireCall?.type === "CallExpression" &&
    requireCall.callee.type === "Identifier" &&
    requireCall.callee.name === "require" &&
    FRAMEWORK_MODULES.has(requireCall.arguments[0]?.value)
  ) {
    return requireCall.arguments[0].value;
  }
  return null;
}

function frameworkBindingModule(node, context) {
  if (node?.type === "MemberExpression" && !node.computed) {
    const module = frameworkBindingModule(node.object, context);
    const prop = propertyName(node.property);
    if (module === "vitest" && prop === "vi") return module;
    if (module === "@jest/globals" && prop === "jest") return module;
    return null;
  }
  if (node?.type !== "Identifier") return null;
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === node.name);
    if (!variable) {
      scope = scope.upper;
      continue;
    }
    for (const def of variable.defs) {
      const importModule = def.type === "ImportBinding" ? def.parent?.source?.value : null;
      if (FRAMEWORK_MODULES.has(importModule) && frameworkImportMatches(def, importModule)) {
        return importModule;
      }
      const requireModule = frameworkRequireModule(def);
      if (requireModule) return requireModule;
    }
    if (variable.defs.length === 0 && node.name === "vi") return "vitest";
    if (variable.defs.length === 0 && node.name === "jest") return "@jest/globals";
    return null;
  }
  if (node.name === "vi") return "vitest";
  if (node.name === "jest") return "@jest/globals";
  return null;
}

function isFrameworkBinding(node, context) {
  return Boolean(frameworkBindingModule(node, context));
}

function frameworkImportMatches(def, importModule) {
  if (def.node?.type === "ImportNamespaceSpecifier") return true;
  const imported = def.node?.imported;
  const name = imported?.type === "Literal" ? String(imported.value) : imported?.name;
  if (importModule === "vitest") return name === "vi";
  return name === "jest";
}

function expressionName(node) {
  if (node?.type === "Identifier") return node.name;
  if (node?.type !== "MemberExpression" || node.computed) return null;
  const object = expressionName(node.object);
  const prop = propertyName(node.property);
  return object && prop ? `${object}.${prop}` : null;
}

module.exports = {
  expressionName,
  frameworkBindingModule,
  isFrameworkBinding,
};
