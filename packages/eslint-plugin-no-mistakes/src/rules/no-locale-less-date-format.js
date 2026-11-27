"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { stableConstInitializer } = require("./async-target-bindings");
const { literalString } = require("./logical-call");
const { globalResolution, isDynamicScope } = require("./no-locale-less-date-format-bindings");

const id = "no-locale-less-date-format";
const DEFAULT_METHODS = ["toLocaleString", "toLocaleDateString", "toLocaleTimeString"];

function resolveConstExpression(node, context) {
  let expression = unwrapExpression(node);
  const seen = new Set();
  for (;;) {
    const initializer = stableConstInitializer(expression, context, seen);
    if (!initializer) return expression;
    expression = unwrapExpression(initializer);
  }
}

function methodName(member, context) {
  if (!member.computed) {
    return member.property.type === "Identifier" ? member.property.name : null;
  }
  return literalString(resolveConstExpression(member.property, context));
}

function isImmediateResolvedOptionsCall(node, context) {
  const expr = unwrapTransparentParent(node);
  const member = expr.parent;
  if (
    member?.type !== "MemberExpression" ||
    member.object !== expr ||
    methodName(member, context) !== "resolvedOptions"
  ) {
    return false;
  }
  const call = unwrapTransparentParent(member);
  return call.parent?.type === "CallExpression" && call.parent.callee === call;
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
          methods: { type: "array", items: { type: "string", pattern: "\\S" }, uniqueItems: true },
          constructors: {
            type: "array",
            items: { type: "string", pattern: "^[^.\\s]+\\.[^.\\s]+$" },
            uniqueItems: true,
          },
          allowUndefinedLocale: { type: "boolean" },
          unknownValues: { enum: ["ignore", "finding"] },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      explicitLocale:
        "Pass an explicit locale to {{method}} so formatting does not depend on the server or browser default locale.",
      unknownLocale:
        "Cannot statically prove {{method}} receives an explicit locale. Pass an explicit locale argument instead of relying on dynamic spread, computed constructor lookup, or dynamic scope.",
      unanalyzable:
        "Cannot tell whether {{method}} is the configured constructor. Its namespace is an import or require this file cannot read, so a missing locale is not proved.",
    },
  },
  (context) => {
    const options = context.options[0] ?? {};
    const methods = new Set(options.methods ?? DEFAULT_METHODS);
    const constructors = new Set(options.constructors ?? []);
    const constructorNamespaces = new Set([...constructors].map((entry) => entry.split(".")[0]));
    const reportUnknown = options.unknownValues === "finding";
    const { sourceCode } = context;
    const suppressed = ruleSuppression(context, id);
    const evalScopes = new Set();
    const pending = [];
    function report(node, method, messageId = "explicitLocale") {
      if (!suppressed(node)) context.report({ node, messageId, data: { method } });
    }

    function checkLocale(node, method, { namespace, usesAlias } = {}) {
      const firstArg = node.arguments[0];
      if (firstArg?.type === "SpreadElement") {
        if (reportUnknown) {
          pending.push({ node, method, namespace, usesAlias, messageId: "unknownLocale" });
        }
        return;
      }
      const locale = unwrapExpression(firstArg);
      const undefinedIdentifier =
        locale?.type === "Identifier" && locale.name === "undefined" ? locale : null;
      const voidLocale = locale?.type === "UnaryExpression" && locale.operator === "void";
      if (locale && (options.allowUndefinedLocale || !(undefinedIdentifier || voidLocale))) return;
      if (namespace || undefinedIdentifier || usesAlias) {
        pending.push({
          node,
          method,
          namespace,
          undefinedIdentifier,
          usesAlias,
          messageId: "explicitLocale",
        });
      } else {
        report(node, method);
      }
    }

    function checkConstructor(node, original) {
      const callee = resolveConstExpression(original, context);
      if (callee.type !== "MemberExpression") return;
      const member = methodName(callee, context);
      const namespace = unwrapExpression(callee.object);
      if (namespace.type !== "Identifier") return;
      if (member === null) {
        if (reportUnknown && callee.computed && constructorNamespaces.has(namespace.name)) {
          pending.push({ node, method: namespace.name, namespace, messageId: "unknownLocale" });
        }
        return;
      }
      const name = `${namespace.name}.${member}`;
      if (!constructors.has(name)) return;
      if (
        namespace.name === "Intl" &&
        member.endsWith("Format") &&
        isImmediateResolvedOptionsCall(node, context)
      ) {
        return;
      }
      checkLocale(node, name, { namespace, usesAlias: callee !== original });
    }

    return {
      CallExpression(node) {
        const callee = unwrapExpression(node.callee);
        if (callee.type === "Identifier" && callee.name === "eval" && !node.optional) {
          const scope = sourceCode.getScope(node);
          if (!scope.isStrict) evalScopes.add(scope.variableScope);
        }
        if (callee.type === "MemberExpression") {
          const method = methodName(callee, context);
          if (methods.has(method)) {
            checkLocale(node, method, {
              usesAlias: callee.computed && unwrapExpression(callee.property).type === "Identifier",
            });
            return;
          }
        }
        if (constructors.size > 0) checkConstructor(node, callee);
      },
      NewExpression(node) {
        if (constructors.size === 0) return;
        checkConstructor(node, unwrapExpression(node.callee));
      },
      "Program:exit"() {
        for (const item of pending) {
          const namespaceState = item.namespace
            ? globalResolution(item.namespace, sourceCode, evalScopes)
            : "global";
          if (namespaceState === "module") {
            if (reportUnknown) report(item.node, item.method, "unanalyzable");
            continue;
          }
          if (namespaceState === "local") continue;
          const undefinedState = item.undefinedIdentifier
            ? globalResolution(item.undefinedIdentifier, sourceCode, evalScopes)
            : "global";
          if (undefinedState === "local") continue;
          let aliasScope = item.usesAlias ? sourceCode.getScope(item.node) : null;
          while (aliasScope && !isDynamicScope(aliasScope, evalScopes))
            aliasScope = aliasScope.upper;
          if (namespaceState === "unknown" || undefinedState === "unknown" || aliasScope) {
            if (reportUnknown) report(item.node, item.method, "unknownLocale");
            continue;
          }
          report(item.node, item.method, item.messageId);
        }
      },
    };
  },
);
