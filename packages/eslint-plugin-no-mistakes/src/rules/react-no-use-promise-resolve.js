"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { literalString } = require("./module-mock-helpers");
const { resolveVariable } = require("./test-no-shared-state-aliases");
const { isGlobalPromiseAlias } = require("./react-no-use-promise-resolve-alias");

function unwrap(node) {
  while (node?.type === "ChainExpression") node = node.expression;
  if (node?.type === "SequenceExpression" && node.expressions?.length) {
    return unwrap(node.expressions.at(-1));
  }
  return node;
}

function staticName(node, computed) {
  if (!node) return null;
  if (computed && node.type === "Identifier") return null;
  if (!computed && typeof node.name === "string" && node.type !== "Literal") return node.name;
  return literalString(node);
}

function variableDef(node, context) {
  const variable = node?.type === "Identifier" ? resolveVariable(node, context) : null;
  return variable?.defs.length === 1 ? variable.defs[0] : null;
}

function reactImport(def) {
  return def?.type === "ImportBinding" && def.parent?.source?.value === "react";
}

function isReactNamespace(node, context, seen = new Set()) {
  const current = unwrap(node);
  if (current?.type !== "Identifier" || seen.has(current.name)) return false;
  const def = variableDef(current, context);
  if (!def) return false;
  if (reactImport(def)) {
    return (
      def.node.type === "ImportNamespaceSpecifier" || def.node.type === "ImportDefaultSpecifier"
    );
  }
  if (def.type !== "Variable" || def.node?.parent?.kind !== "const" || !def.node.init) return false;
  if (def.node.id.type !== "Identifier") return false;
  seen.add(current.name);
  return isReactNamespace(def.node.init, context, seen);
}

function patternKey(pattern, name) {
  if (pattern?.type !== "ObjectPattern") return null;
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    let value = property.value;
    while (value?.type === "AssignmentPattern") value = value.left;
    if (value?.type === "Identifier" && value.name === name) {
      return property.computed ? literalString(property.key) : staticName(property.key, false);
    }
  }
  return null;
}

function isReactUse(node, context, useNames, seen = new Set()) {
  let callee = unwrap(node);
  if (callee?.type === "MemberExpression") {
    const name = staticName(callee.property, callee.computed);
    if (name === "call" || name === "apply") callee = unwrap(callee.object);
  }
  if (callee?.type === "Identifier") {
    if (seen.has(callee.name)) return false;
    const def = variableDef(callee, context);
    if (def?.type === "ImportBinding") return useNames.has(callee.name);
    if (def?.type === "Variable" && def.node?.parent?.kind === "const" && def.node.init) {
      seen.add(callee.name);
      if (def.node.id.type === "Identifier")
        return isReactUse(def.node.init, context, useNames, seen);
      return (
        patternKey(def.node.id, callee.name) === "use" &&
        isReactNamespace(def.node.init, context, seen)
      );
    }
    if (def) return false;
    return useNames.has(callee.name);
  }
  return (
    callee?.type === "MemberExpression" &&
    staticName(callee.property, callee.computed) === "use" &&
    isReactNamespace(callee.object, context)
  );
}

function isPromiseResolve(node) {
  const current = unwrap(node);
  if (current?.type !== "CallExpression") return false;
  let callee = unwrap(current.callee);
  if (
    callee?.type === "MemberExpression" &&
    (staticName(callee.property, callee.computed) === "call" ||
      staticName(callee.property, callee.computed) === "apply")
  ) {
    callee = unwrap(callee.object);
  }
  return (
    callee?.type === "MemberExpression" &&
    unwrap(callee.object)?.type === "Identifier" &&
    unwrap(callee.object).name === "Promise" &&
    staticName(callee.property, callee.computed) === "resolve"
  );
}

function isAliasedPromiseResolve(node, context) {
  if (
    node?.type !== "CallExpression" ||
    node.optional ||
    node.arguments.length !== 1 ||
    node.arguments[0].type === "SpreadElement"
  )
    return false;
  const callee = node.callee;
  return (
    callee.type === "MemberExpression" &&
    !callee.optional &&
    callee.object.type === "Identifier" &&
    staticName(callee.property, callee.computed) === "resolve" &&
    isGlobalPromiseAlias(callee.object, context)
  );
}

function ordinaryUseCall(node) {
  if (node.optional || node.arguments.length !== 1 || node.arguments[0].type === "SpreadElement")
    return false;
  const callee = node.callee;
  return (
    callee.type === "Identifier" ||
    (callee.type === "MemberExpression" &&
      !callee.optional &&
      !["call", "apply"].includes(staticName(callee.property, callee.computed)))
  );
}

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: { description: "disallow React.use(Promise.resolve(...))", recommended: false },
      schema: [],
      messages: {
        resolve: "Avoid React.use(Promise.resolve(...)); pass the promise directly or use await.",
      },
    },
    (context) => {
      const suppressed = ruleSuppression(context, "react-no-use-promise-resolve");
      const useNames = new Set();
      return {
        ImportDeclaration(node) {
          if (node.source.value !== "react") return;
          for (const specifier of node.specifiers) {
            if (specifier.type !== "ImportSpecifier") continue;
            if (staticName(specifier.imported, false) === "use") useNames.add(specifier.local.name);
          }
        },
        CallExpression(node) {
          const forwarded = unwrap(node.callee);
          const forwarding =
            forwarded?.type === "MemberExpression" &&
            (staticName(forwarded.property, forwarded.computed) === "call" ||
              staticName(forwarded.property, forwarded.computed) === "apply");
          const resolvedArgument = forwarding
            ? staticName(forwarded.property, false) === "apply"
              ? node.arguments[1]?.elements?.[0]
              : node.arguments[1]
            : node.arguments[0];
          if (
            suppressed(node) ||
            !isReactUse(node.callee, context, useNames) ||
            (!isPromiseResolve(resolvedArgument) &&
              !(ordinaryUseCall(node) && isAliasedPromiseResolve(node.arguments[0], context)))
          ) {
            return;
          }
          context.report({ node, messageId: "resolve" });
        },
      };
    },
  ),
  {
    __test: { isPromiseResolve, isReactNamespace, isReactUse, patternKey, staticName },
  },
);
