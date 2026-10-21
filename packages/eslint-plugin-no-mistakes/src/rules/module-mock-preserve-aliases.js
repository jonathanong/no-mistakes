"use strict";

const { collectPatternNames } = require("./ast-pattern-names");
const { unwrapExpression, unwrapSequence } = require("./async-ast");
const {
  frameworkBindingModule,
  expressionName,
  isFrameworkBinding,
  memberPropertyName,
  propertyName,
} = require("./module-mock-helpers");

const PRESERVE_METHODS = new Set(["mock", "doMock", "unstable_mockModule"]);

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

function bindingOrigin(id) {
  const parent = id?.parent;
  if (parent?.type === "VariableDeclarator" || parent?.type === "AssignmentExpression") {
    return parent;
  }
  return id;
}

function writtenAfter(variable, origin, use) {
  if (!variable?.references || !origin?.range || !use?.range) return false;
  return variable.references.some(
    (reference) =>
      reference.isWrite() &&
      reference.identifier.range[0] >= origin.range[1] &&
      reference.identifier.range[0] < use.range[0],
  );
}

function createMockAliases(context, methods) {
  const aliases = new Map();

  function record(name, variable, mock, origin) {
    const entries = aliases.get(name) ?? [];
    entries.push({ mock, origin, variable });
    aliases.set(name, entries);
  }

  function recordPattern(pattern, mock, origin) {
    if (pattern.type === "Identifier") {
      record(pattern.name, resolveVariable(pattern, context), mock, origin);
      return;
    }
    for (const name of collectPatternNames(pattern)) {
      record(name, resolveVariableByName(name, pattern, context), mock, origin);
    }
  }

  return {
    declareImport(local, source, imported) {
      if (!methods.has(imported)) return;
      record(
        local.name,
        resolveVariable(local, context),
        {
          framework: source,
          method: imported,
          namespace: source === "vitest" ? "vi" : "jest",
        },
        local,
      );
    },
    declare(id, init) {
      if (!id || !init) return;
      init = unwrapExpression(init);
      const origin = bindingOrigin(id);
      if (id.type === "ObjectPattern" && isFrameworkBinding(init, context)) {
        for (const property of id.properties) {
          if (property.type !== "Property") continue;
          if (property.computed && property.key?.type === "Identifier") continue;
          const method = propertyName(property.key);
          if (!methods.has(method)) continue;
          recordPattern(property.value, frameworkMock(init, method, context), origin);
        }
      }
      if (init.type === "MemberExpression" && isFrameworkBinding(init.object, context)) {
        const method = memberPropertyName(init);
        if (!methods.has(method)) return;
        recordPattern(id, frameworkMock(init.object, method, context), origin);
      }
    },
    get(node) {
      if (node.type !== "Identifier") return null;
      const entries = aliases.get(node.name);
      if (!entries) return null;
      const variable = resolveVariable(node, context);
      if (!node.range) {
        return entries.find((candidate) => candidate.variable === variable)?.mock ?? null;
      }
      const entry = entries.findLast(
        (candidate) =>
          candidate.variable === variable &&
          candidate.origin?.range &&
          candidate.origin.range[1] <= node.range[0],
      );
      if (!entry || writtenAfter(variable, entry.origin, node)) return null;
      return entry.mock;
    },
    matchCall(node) {
      const callee = unwrapSequence(node.callee);
      if (callee?.type === "Identifier") {
        const direct = this.get(callee);
        if (direct) return { mock: direct };
      }
      if (
        node.callee?.type !== "MemberExpression" ||
        !["call", "apply"].includes(memberPropertyName(node.callee))
      ) {
        return null;
      }
      const object = unwrapSequence(node.callee.object);
      if (object?.type !== "Identifier") return null;
      const mock = this.get(object);
      if (!mock) return null;
      const args = node.arguments ?? [];
      if (args.some((arg) => arg?.type === "SpreadElement")) return null;
      if (memberPropertyName(node.callee) === "call") {
        return {
          factory: args[2] ?? null,
          forwarded: true,
          mock,
          specifierNode: args[1] ?? null,
        };
      }
      const list = unwrapExpression(args[1]);
      if (
        list?.type === "ArrayExpression" &&
        list.elements.some((element) => !element || element.type === "SpreadElement")
      ) {
        return null;
      }
      return {
        factory: list?.type === "ArrayExpression" ? (list.elements[1] ?? null) : null,
        forwarded: true,
        mock,
        specifierNode: list?.type === "ArrayExpression" ? (list.elements[0] ?? null) : null,
      };
    },
  };
}

function createPreserveMockAliases(context) {
  return createMockAliases(context, PRESERVE_METHODS);
}

module.exports = {
  createMockAliases,
  createPreserveMockAliases,
  resolveVariable,
};
