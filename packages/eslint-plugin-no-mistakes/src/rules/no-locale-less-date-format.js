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

function isGlobalUndefined(identifier, sourceCode, evalScopes) {
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
      description: "require explicit locales for configured formatting methods",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          methods: { type: "array", items: { type: "string" }, uniqueItems: true },
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
    const { sourceCode } = context;
    const evalScopes = new Set();
    const undefinedLocales = [];
    function report(node, method) {
      context.report({ node, messageId: "explicitLocale", data: { method } });
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
        if (!methods.has(method)) return;
        const locale = unwrapExpression(node.arguments[0]);
        if (!locale) {
          report(node, method);
        } else if (!options.allowUndefinedLocale) {
          if (locale.type === "UnaryExpression" && locale.operator === "void") {
            report(node, method);
          } else if (locale.type === "Identifier" && locale.name === "undefined") {
            undefinedLocales.push({ node, method, locale });
          }
        }
      },
      "Program:exit"() {
        for (const { node, method, locale } of undefinedLocales) {
          if (isGlobalUndefined(locale, sourceCode, evalScopes)) report(node, method);
        }
      },
    };
  },
);
