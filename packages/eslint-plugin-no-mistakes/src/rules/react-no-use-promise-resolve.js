"use strict";

const { rule } = require("../helpers");

function isImportedReactUse(node, context) {
  let scope = context.sourceCode.getScope(node);
  let variable = null;
  while (scope && !variable) {
    variable = scope.variables.find((scopeVariable) => scopeVariable.name === node.name);
    scope = scope.upper;
  }
  return Boolean(
    variable?.defs.some(
      (def) => def.type === "ImportBinding" && def.parent?.source?.value === "react",
    ),
  );
}

function isImportedFromReact(node, context) {
  let scope = context.sourceCode.getScope(node);
  let variable = null;
  while (scope && !variable) {
    variable = scope.variables.find((scopeVariable) => scopeVariable.name === node.name);
    scope = scope.upper;
  }
  return Boolean(
    variable?.defs.some(
      (def) =>
        (def.type === "ImportBinding" || def.type === "Variable") &&
        def.parent?.source?.value === "react",
    ),
  );
}

function propertyName(node) {
  if (node.type === "Literal") return String(node.value);
  return node.name;
}

function isReactUse(callee, context, useNames) {
  if (callee?.type === "Identifier") {
    return useNames.has(callee.name) && isImportedReactUse(callee, context);
  }
  return (
    callee?.type === "MemberExpression" &&
    callee.object?.type === "Identifier" &&
    isImportedFromReact(callee.object, context) &&
    propertyName(callee.property) === "use"
  );
}

function isPromiseResolve(node) {
  return (
    node?.type === "CallExpression" &&
    node.callee?.type === "MemberExpression" &&
    node.callee.object?.type === "Identifier" &&
    node.callee.object.name === "Promise" &&
    propertyName(node.callee.property) === "resolve"
  );
}

module.exports = rule(
  {
    type: "problem",
    docs: { description: "disallow React.use(Promise.resolve(...))", recommended: false },
    schema: [],
    messages: {
      resolve: "Avoid React.use(Promise.resolve(...)); pass the promise directly or use await.",
    },
  },
  (context) => {
    const useNames = new Set();
    return {
      ImportDeclaration(node) {
        if (node.source.value !== "react") return;
        for (const specifier of node.specifiers) {
          if (
            specifier.type === "ImportSpecifier" &&
            specifier.imported.type === "Identifier" &&
            specifier.imported.name === "use"
          ) {
            useNames.add(specifier.local.name);
          }
        }
      },
      CallExpression(node) {
        if (!isReactUse(node.callee, context, useNames) || !isPromiseResolve(node.arguments[0])) {
          return;
        }
        context.report({ node, messageId: "resolve" });
      },
    };
  },
);
