"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");

const id = "no-locale-less-date-format";
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

function isImmediateResolvedOptionsCall(node) {
  const expr = unwrapTransparentParent(node);
  const member = expr.parent;
  if (
    member?.type !== "MemberExpression" ||
    member.object !== expr ||
    methodName(member) !== "resolvedOptions"
  ) {
    return false;
  }
  const call = unwrapTransparentParent(member);
  return call.parent?.type === "CallExpression" && call.parent.callee === call;
}

function globalResolution(identifier, sourceCode, evalScopes) {
  let reference;
  for (let scope = sourceCode.getScope(identifier); scope; scope = scope.upper) {
    if (scope.type === "with" || scope.directCallToEvalScope || evalScopes.has(scope)) {
      return "unknown";
    }
    reference ??= scope.references.find((item) => item.identifier === identifier);
  }
  const variable = reference?.resolved;
  return variable?.scope?.type === "global" && variable.defs.length === 0 ? "global" : "local";
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

    function checkLocale(node, method, namespace) {
      const firstArg = node.arguments[0];
      if (firstArg?.type === "SpreadElement") {
        if (reportUnknown) {
          if (namespace) pending.push({ node, method, namespace, messageId: "unknownLocale" });
          else report(node, method, "unknownLocale");
        }
        return;
      }
      const locale = unwrapExpression(firstArg);
      const undefinedIdentifier =
        locale?.type === "Identifier" && locale.name === "undefined" ? locale : null;
      const voidLocale = locale?.type === "UnaryExpression" && locale.operator === "void";
      if (locale && (options.allowUndefinedLocale || !(undefinedIdentifier || voidLocale))) return;
      if (namespace || undefinedIdentifier) {
        pending.push({ node, method, namespace, undefinedIdentifier, messageId: "explicitLocale" });
      } else {
        report(node, method);
      }
    }

    function checkConstructor(node, callee, member) {
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
        isImmediateResolvedOptionsCall(node)
      ) {
        return;
      }
      checkLocale(node, name, namespace);
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
        for (const item of pending) {
          const namespaceState = item.namespace
            ? globalResolution(item.namespace, sourceCode, evalScopes)
            : "global";
          if (namespaceState === "local") continue;
          const undefinedState = item.undefinedIdentifier
            ? globalResolution(item.undefinedIdentifier, sourceCode, evalScopes)
            : "global";
          if (undefinedState === "local") continue;
          if (namespaceState === "unknown" || undefinedState === "unknown") {
            if (reportUnknown) report(item.node, item.method, "unknownLocale");
            continue;
          }
          report(item.node, item.method, item.messageId);
        }
      },
    };
  },
);
