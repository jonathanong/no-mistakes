"use strict";

const { rule } = require("../helpers");
const { unwrapExpression } = require("./async-ast");

const DEFAULT_METHODS = ["toLocaleString", "toLocaleDateString", "toLocaleTimeString"];

function methodName(member) {
  if (!member.computed) {
    return member.property.type === "Identifier" ? member.property.name : null;
  }
  const key = member.property;
  if (key.type === "Literal" && typeof key.value === "string") return key.value;
  if (key.type === "TemplateLiteral" && key.expressions.length === 0) {
    return key.quasis[0].value.cooked;
  }
  return null;
}

function isUnshadowedGlobal(identifier, sourceCode, evalScopes) {
  let reference;
  for (let scope = sourceCode.getScope(identifier); scope; scope = scope.upper) {
    if (scope.type === "with" || scope.directCallToEvalScope || evalScopes.has(scope)) return false;
    reference ??= scope.references.find((item) => item.identifier === identifier);
  }
  const variable = reference?.resolved;
  return variable?.scope?.type === "global" && variable.defs.length === 0;
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require explicit locales for configured formatting methods and constructors",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          methods: { type: "array", items: { type: "string" }, uniqueItems: true },
          constructors: { type: "array", items: { type: "string" }, uniqueItems: true },
          allowUndefinedLocale: { type: "boolean" },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      explicitLocale:
        "Pass an explicit locale to {{method}} so formatting does not depend on the server or browser default locale.",
    },
  },
  (context) => {
    const options = context.options[0] ?? {};
    const methods = new Set(options.methods ?? DEFAULT_METHODS);
    const constructors = new Set(options.constructors ?? []);
    const { sourceCode } = context;
    const evalScopes = new Set();
    const pending = [];
    function report(node, method) {
      context.report({ node, messageId: "explicitLocale", data: { method } });
    }

    function checkLocale(node, method, namespace) {
      const locale = unwrapExpression(node.arguments[0]);
      const undefinedIdentifier =
        locale?.type === "Identifier" && locale.name === "undefined" ? locale : null;
      const voidLocale = locale?.type === "UnaryExpression" && locale.operator === "void";
      if (locale && (options.allowUndefinedLocale || !(undefinedIdentifier || voidLocale))) return;
      if (namespace || undefinedIdentifier) {
        pending.push({ node, method, namespace, undefinedIdentifier });
      } else {
        report(node, method);
      }
    }

    function checkConstructor(node, callee, member) {
      if (member === null) return;
      const namespace = unwrapExpression(callee.object);
      if (namespace.type !== "Identifier") return;
      const name = `${namespace.name}.${member}`;
      if (constructors.has(name)) checkLocale(node, name, namespace);
    }

    return {
      CallExpression(node) {
        const callee = unwrapExpression(node.callee);
        if (callee.type === "Identifier" && callee.name === "eval" && !node.optional) {
          const scope = sourceCode.getScope(node);
          if (!scope.isStrict) evalScopes.add(scope.variableScope);
        }
        if (callee.type !== "MemberExpression") return;
        const method = methodName(callee);
        if (methods.has(method)) {
          checkLocale(node, method);
        } else if (constructors.size > 0) {
          checkConstructor(node, callee, method);
        }
      },
      NewExpression(node) {
        if (constructors.size === 0) return;
        const callee = unwrapExpression(node.callee);
        if (callee.type === "MemberExpression") checkConstructor(node, callee, methodName(callee));
      },
      "Program:exit"() {
        for (const { node, method, namespace, undefinedIdentifier } of pending) {
          if (namespace && !isUnshadowedGlobal(namespace, sourceCode, evalScopes)) continue;
          if (
            undefinedIdentifier &&
            !isUnshadowedGlobal(undefinedIdentifier, sourceCode, evalScopes)
          ) {
            continue;
          }
          report(node, method);
        }
      },
    };
  },
);
