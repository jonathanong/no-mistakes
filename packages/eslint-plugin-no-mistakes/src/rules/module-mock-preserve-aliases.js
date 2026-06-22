"use strict";

const { collectPatternNames } = require("./ast-pattern-names");
const {
  frameworkBindingModule,
  expressionName,
  isFrameworkBinding,
  memberPropertyName,
  propertyName,
} = require("./module-mock-helpers");

function resolveVariable(node, context) {
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === node.name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function resolveVariableByName(name, node, context) {
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function frameworkMock(object, method, context) {
  const framework = frameworkBindingModule(object, context);
  return { framework, method, namespace: expressionName(object) };
}

function createPreserveMockAliases(context) {
  const aliases = new Map();

  function record(name, variable, mock) {
    const entries = aliases.get(name) ?? [];
    entries.push({ mock, variable });
    aliases.set(name, entries);
  }

  function recordPattern(pattern, mock) {
    if (pattern.type === "Identifier") {
      record(pattern.name, resolveVariable(pattern, context), mock);
      return;
    }
    for (const name of collectPatternNames(pattern)) {
      record(name, resolveVariableByName(name, pattern, context), mock);
    }
  }

  return {
    declareImport(local, source, imported) {
      if (imported !== "mock" && imported !== "doMock" && imported !== "unstable_mockModule") {
        return;
      }
      record(local.name, resolveVariable(local, context), {
        framework: source,
        method: imported,
        namespace: source === "vitest" ? "vi" : "jest",
      });
    },
    declare(id, init) {
      if (!id || !init) return;
      if (id.type === "ObjectPattern" && isFrameworkBinding(init, context)) {
        for (const property of id.properties) {
          if (property.type !== "Property") continue;
          const method = propertyName(property.key);
          if (method !== "mock" && method !== "doMock" && method !== "unstable_mockModule") {
            continue;
          }
          recordPattern(property.value, frameworkMock(init, method, context));
        }
      }
      if (init.type === "MemberExpression" && isFrameworkBinding(init.object, context)) {
        const method = memberPropertyName(init);
        if (method !== "mock" && method !== "doMock" && method !== "unstable_mockModule") return;
        recordPattern(id, frameworkMock(init.object, method, context));
      }
    },
    get(node) {
      if (node.type !== "Identifier") return null;
      const entries = aliases.get(node.name);
      if (!entries) return null;
      const variable = resolveVariable(node, context);
      return entries.find((entry) => entry.variable === variable)?.mock ?? null;
    },
    matchCall(node) {
      const direct = this.get(node.callee);
      if (direct) return { mock: direct };
      if (
        node.callee.type !== "MemberExpression" ||
        node.callee.object.type !== "Identifier" ||
        !["call", "apply"].includes(memberPropertyName(node.callee))
      ) {
        return null;
      }
      const mock = this.get(node.callee.object);
      if (!mock) return null;
      if (memberPropertyName(node.callee) === "call") {
        return { factory: node.arguments[2], mock, specifierNode: node.arguments[1] };
      }
      const args = node.arguments[1];
      return {
        factory: args?.type === "ArrayExpression" ? args.elements[1] : undefined,
        mock,
        specifierNode: args?.type === "ArrayExpression" ? args.elements[0] : undefined,
      };
    },
  };
}

module.exports = {
  createPreserveMockAliases,
  resolveVariable,
};
