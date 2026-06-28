"use strict";

const { rule } = require("../helpers");
const {
  memberPropertyName,
  propertyName,
  repoRelativeFilename,
  stringMatches,
} = require("./module-mock-helpers");

const GLOBAL_FETCH_ROOTS = new Set(["globalThis", "window", "self", "global"]);
const LOCAL_BINDING_TYPES = new Set([
  "Variable",
  "Parameter",
  "CatchClause",
  "FunctionName",
  "ImportBinding",
  "ClassName",
]);

function variableFromScope(scope, name) {
  const get = scope?.set?.get;
  if (typeof get === "function") return get.call(scope.set, name) ?? null;
  return scope?.variables?.find((item) => item.name === name) ?? null;
}

function resolveVariable(node, context) {
  if (node?.type !== "Identifier") return null;
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = variableFromScope(scope, node.name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function hasLocalBinding(node, context) {
  const variable = resolveVariable(node, context);
  return Boolean(variable?.defs?.some((def) => LOCAL_BINDING_TYPES.has(def.type)));
}

function unwrapTSAndChain(node) {
  while (
    node &&
    (node.type === "ChainExpression" ||
      node.type === "TSAsExpression" ||
      node.type === "TSSatisfiesExpression" ||
      node.type === "TSNonNullExpression" ||
      node.type === "TSInstantiationExpression" ||
      node.type === "TSTypeAssertion")
  ) {
    node = node.expression;
  }
  return node;
}

function isUnshadowedGlobalRoot(node, context) {
  const unwrapped = unwrapTSAndChain(node);
  return (
    unwrapped?.type === "Identifier" &&
    GLOBAL_FETCH_ROOTS.has(unwrapped.name) &&
    !hasLocalBinding(unwrapped, context)
  );
}

function fetchMemberProperty(node) {
  return memberPropertyName(node) === "fetch";
}

function isGlobalFetchMember(node, context) {
  const unwrapped = unwrapTSAndChain(node);
  return (
    unwrapped?.type === "MemberExpression" &&
    fetchMemberProperty(unwrapped) &&
    isUnshadowedGlobalRoot(unwrapped.object, context)
  );
}

function isGlobalFetchExpression(node, context, aliases) {
  const unwrapped = unwrapTSAndChain(node);
  if (!unwrapped) return false;
  if (unwrapped.type === "Identifier") {
    const variable = resolveVariable(unwrapped, context);
    if (variable && aliases.has(variable)) return true;
    if (unwrapped.name === "fetch") return !hasLocalBinding(unwrapped, context);
    return false;
  }
  return isGlobalFetchMember(unwrapped, context);
}

function bindingIdentifier(node) {
  if (node?.type === "Identifier") return node;
  return node?.type === "AssignmentPattern" && node.left.type === "Identifier" ? node.left : null;
}

function recordObjectPatternFetchAliases(id, init, context, aliases) {
  if (id.type !== "ObjectPattern" || !isUnshadowedGlobalRoot(init, context)) return;
  for (const property of id.properties) {
    if (property.type !== "Property" || propertyName(property.key) !== "fetch") continue;
    const binding = bindingIdentifier(property.value);
    const variable = resolveVariable(binding, context);
    if (variable) aliases.add(variable);
  }
}

function childNodes(node) {
  const children = [];
  for (const [key, value] of Object.entries(node)) {
    if (key === "parent") continue;
    if (Array.isArray(value)) {
      for (const item of value) {
        if (item?.type) children.push(item);
      }
    } else if (value?.type) {
      children.push(value);
    }
  }
  return children;
}

function collectVariableDeclarators(node, declarators = []) {
  if (node.type === "VariableDeclarator") declarators.push(node);
  for (const child of childNodes(node)) collectVariableDeclarators(child, declarators);
  return declarators;
}

function collectFetchAliases(program, context, aliases) {
  const declarators = collectVariableDeclarators(program);
  let changed = true;
  while (changed) {
    changed = false;
    for (const node of declarators) {
      if (node.parent?.type !== "VariableDeclaration" || node.parent.kind !== "const") continue;
      const before = aliases.size;
      recordObjectPatternFetchAliases(node.id, node.init, context, aliases);
      if (node.id.type === "Identifier" && isGlobalFetchExpression(node.init, context, aliases)) {
        aliases.add(resolveVariable(node.id, context));
      }
      changed ||= aliases.size > before;
    }
  }
}

function shouldCheckFile(filename, options) {
  if (!options) return false;
  const file = repoRelativeFilename(filename);
  return (
    stringMatches(file, options.checkedPathPatterns ?? []) &&
    !stringMatches(file, options.allowedPathPatterns ?? [])
  );
}

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: {
        description: "disallow global fetch outside configured helper paths",
        recommended: false,
      },
      schema: {
        type: "array",
        minItems: 1,
        maxItems: 1,
        items: [
          {
            type: "object",
            properties: {
              checkedPathPatterns: { type: "array", minItems: 1, items: { type: "string" } },
              allowedPathPatterns: { type: "array", minItems: 1, items: { type: "string" } },
            },
            required: ["checkedPathPatterns", "allowedPathPatterns"],
            additionalProperties: false,
          },
        ],
      },
      messages: {
        globalFetch:
          "Move global fetch calls into a configured API/client helper so request behavior stays centralized.",
      },
    },
    (context) => {
      const options = context.options[0];
      if (!shouldCheckFile(context.filename, options)) return {};
      const aliases = new Set();

      return {
        Program(node) {
          collectFetchAliases(node, context, aliases);
        },
        CallExpression(node) {
          if (!isGlobalFetchExpression(node.callee, context, aliases)) return;
          context.report({ node: node.callee, messageId: "globalFetch" });
        },
      };
    },
  ),
  {
    __test: {
      bindingIdentifier,
      childNodes,
      collectFetchAliases,
      collectVariableDeclarators,
      hasLocalBinding,
      isGlobalFetchExpression,
      isGlobalFetchMember,
      shouldCheckFile,
      unwrapTSAndChain,
    },
  },
);
