"use strict";

const { unwrapExpression } = require("./async-ast");
const { opaqueIdentifierAlias, opaqueImportEquals } = require("./module-mock-framework-opaque");

function staticImportSource(definition) {
  return require("./async-target-bindings").staticImportSource(definition);
}

function requireSource(node, context) {
  return require("./async-target-sources").requireSource(node, context);
}

function requireResolution(node, context) {
  return require("./async-target-sources").requireResolution(node, context);
}

const FRAMEWORK_MODULES = new Set(["vitest", "@jest/globals"]);

function propertyName(node) {
  node = unwrapExpression(node);
  if (!node) return null;
  if (node.type === "Literal") return String(node.value);
  if (typeof node.name === "string") return node.name;
  if (node.type === "TemplateLiteral" && node.expressions.length === 0) {
    return node.quasis.map((quasi) => quasi.value.cooked ?? quasi.value.raw).join("");
  }
  return null;
}

function memberPropertyName(node) {
  const current = unwrapExpression(node);
  if (current?.type !== "MemberExpression") return null;
  const property = unwrapExpression(current.property);
  if (current.computed && property?.type === "Identifier") return null;
  return propertyName(property);
}

function frameworkRequireSource(init, context) {
  let call = unwrapExpression(init);
  const member = call?.type === "MemberExpression" ? memberPropertyName(call) : null;
  if (member) call = unwrapExpression(call.object);
  let source = requireSource(call, context);
  const argument = call?.type === "CallExpression" ? call.arguments?.[0] : null;
  if (source === null && argument && argument.type == null && typeof argument.value === "string") {
    source = argument.value;
  }
  if (!member) return source;
  if (source === "vitest" && member === "vi") return source;
  if (source === "@jest/globals" && member === "jest") return source;
  return null;
}

function frameworkRequireModule(def, name, context) {
  if (def.type !== "Variable") return null;
  const source = frameworkRequireSource(def.node?.init, context);
  if (!FRAMEWORK_MODULES.has(source)) return null;
  if (def.node?.id?.type !== "ObjectPattern") return source;
  const key = patternKey(def.node.id, name);
  if (source === "vitest" && key === "vi") return source;
  if (source === "@jest/globals" && key === "jest") return source;
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
      const importModule = def.type === "ImportBinding" ? staticImportSource(def) : null;
      if (FRAMEWORK_MODULES.has(importModule) && frameworkImportMatches(def, importModule)) {
        return importModule;
      }
      if (def.node?.type === "ImportDefaultSpecifier" && FRAMEWORK_MODULES.has(importModule)) {
        return importModule;
      }
      const requireModule = frameworkRequireModule(def, node.name, context);
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

function frameworkBindingOpaque(node, context, seen = new Set()) {
  node = unwrapExpression(node);
  if (!node || frameworkBindingModule(node, context)) return false;
  if (node.type === "CallExpression") {
    return requireResolution(node, context)?.source === null;
  }
  if (node.type === "MemberExpression") {
    const known = frameworkBindingModule(node.object, context);
    if (known && memberPropertyName(node) === null) return false;
    return frameworkBindingOpaque(node.object, context, seen);
  }
  if (node.type !== "Identifier" || seen.has(node.name)) return false;
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const found = scope.variables.find((candidate) => candidate.name === node.name);
    if (!found) {
      scope = scope.upper;
      continue;
    }
    if (found.defs.length !== 1) return false;
    seen.add(node.name);
    const def = found.defs[0];
    const required = requireResolution(def.node?.init, context);
    if (required?.source === null && def.type === "Variable") return true;
    if (opaqueImportEquals(def)) return true;
    if (def.type === "Variable" && def.node?.id?.type === "ObjectPattern") {
      const key = patternKey(def.node.id, node.name);
      if (key === null && required?.source === null) return true;
    }
    if (opaqueIdentifierAlias(def, context, seen, frameworkBindingOpaque)) return true;
    return false;
  }
  return false;
}

function frameworkImportMatches(def, importModule) {
  if (
    def.node?.type === "ImportNamespaceSpecifier" ||
    def.node?.type === "TSImportEqualsDeclaration"
  ) {
    return true;
  }
  const imported = def.node?.imported;
  const name = imported?.type === "Literal" ? String(imported.value) : imported?.name;
  if (importModule === "vitest") return name === "vi";
  return name === "jest";
}

function patternKey(pattern, name) {
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const key = unwrapExpression(property.key);
    if (property.computed && key?.type === "Identifier") continue;
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
  frameworkBindingOpaque,
  isFrameworkBinding,
};
