"use strict";

const { unwrapExpression } = require("./async-ast");

const FRAMEWORK_MODULES = new Set(["vitest", "@jest/globals"]);

function propertyName(node) {
  if (!node) return null;
  if (node.type === "Literal") return String(node.value);
  if (typeof node.name === "string") return node.name;
  if (node.type === "TemplateLiteral" && node.expressions.length === 0) {
    return node.quasis.map((quasi) => quasi.value.cooked ?? quasi.value.raw).join("");
  }
  return null;
}

function memberPropertyName(node) {
  let current = node;
  while (current?.type === "ChainExpression") current = current.expression;
  if (current?.type !== "MemberExpression") return null;
  if (current.computed && current.property?.type === "Identifier") return null;
  return propertyName(current.property);
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

function frameworkBindingModule(node, context, seen = new Set()) {
  node = unwrapExpression(node);
  if (node?.type === "MemberExpression") {
    const module = frameworkBindingModule(node.object, context, seen);
    const prop = memberPropertyName(node);
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
    if (seen.has(variable)) return null;
    seen.add(variable);
    for (const def of variable.defs) {
      const importModule = def.type === "ImportBinding" ? def.parent?.source?.value : null;
      if (FRAMEWORK_MODULES.has(importModule) && frameworkImportMatches(def, importModule)) {
        return importModule;
      }
      if (def.node?.type === "ImportDefaultSpecifier" && FRAMEWORK_MODULES.has(importModule)) {
        return importModule;
      }
      const requireModule = frameworkRequireModule(def);
      if (requireModule) return requireModule;
      if (
        def.type === "Variable" &&
        def.parent?.kind === "const" &&
        def.node?.id?.type === "Identifier" &&
        def.node.init
      ) {
        const aliased = frameworkBindingModule(def.node.init, context, seen);
        if (aliased) return aliased;
      }
      if (
        def.type === "Variable" &&
        def.parent?.kind === "const" &&
        def.node?.id?.type === "ObjectPattern" &&
        def.node.init
      ) {
        const key = patternKey(def.node.id, node.name);
        const source = frameworkBindingModule(def.node.init, context, seen);
        if (
          (source === "vitest" && key === "vi") ||
          (source === "@jest/globals" && key === "jest")
        ) {
          return source;
        }
      }
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

function patternKey(pattern, name) {
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    if (property.computed && property.key?.type === "Identifier") continue;
    const value =
      property.value.type === "AssignmentPattern" ? property.value.left : property.value;
    if (value.type === "Identifier" && value.name === name) return propertyName(property.key);
  }
  return null;
}

function expressionName(node) {
  while (node?.type === "ChainExpression") node = node.expression;
  if (node?.type === "Identifier") return node.name;
  if (node?.type !== "MemberExpression") return null;
  const object = expressionName(node.object);
  const prop = memberPropertyName(node);
  return object && prop ? `${object}.${prop}` : null;
}

module.exports = {
  expressionName,
  frameworkBindingModule,
  isFrameworkBinding,
};
