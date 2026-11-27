"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { logicalCall, memberPropertyName, pathAllowed } = require("./module-mock-helpers");
const {
  importSpellings,
  reportsUnanalyzable,
  requiredMember,
  rootIdentifier,
} = require("./import-resolution");
const helpers = require("./nullable-fetch-wrapper-helpers");

const {
  calleePath,
  collectExportedNames,
  collectFunctionOverloadReturnTypes,
  collectFunctionTypeReturns,
  compilePatterns,
  functionName,
  functionReturnAnnotation,
  hasCheckedPath,
  isExportedFunction,
  typeMatchesNullableHint,
} = helpers;

function isFunctionBoundary(node) {
  return (
    node.type === "FunctionDeclaration" ||
    node.type === "FunctionExpression" ||
    node.type === "ArrowFunctionExpression"
  );
}

function calleeNames(callee, context) {
  const path = calleePath(callee);
  const spelled = importSpellings(rootIdentifier(callee), path, context);
  const required = requiredMember(callee, context);
  return {
    names: [...new Set([path, ...spelled.names, required?.name].filter(Boolean))],
    unresolved: spelled.unresolved || required?.unresolved === true,
  };
}

function argumentContains(argument, target) {
  return (
    argument?.range &&
    target?.range &&
    argument.range[0] <= target.range[0] &&
    argument.range[1] >= target.range[1]
  );
}

function wrapperState(node, wrapper, context) {
  let current = node.parent;
  let unresolved = false;
  while (current) {
    if (current.type === "CallExpression") {
      const info = calleeNames(current.callee, context);
      const invoked = logicalCall(current);
      const forwarded = invoked ? calleeNames(invoked.callee, context) : null;
      const matched =
        info.names.includes(wrapper) ||
        (forwarded?.names.includes(wrapper) &&
          invoked.arguments.some((argument) => argumentContains(argument, node)));
      if (matched) return "inside";
      const holds =
        current.arguments.some((argument) => argumentContains(argument, node)) ||
        (invoked?.arguments ?? []).some((argument) => argumentContains(argument, node));
      if (holds && (info.unresolved || forwarded?.unresolved)) unresolved = true;
    }
    if (isFunctionBoundary(current)) return unresolved ? "unresolved" : "outside";
    current = current.parent;
  }
  return unresolved ? "unresolved" : "outside";
}

function insideWrapper(node, wrapper, context) {
  return wrapperState(node, wrapper, context) === "inside";
}

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: {
        description: "require nullable entity fetches to use a configured wrapper",
        recommended: false,
      },
      schema: [
        {
          type: "object",
          properties: {
            includePathPatterns: { type: "array", items: { type: "string" } },
            excludePathPatterns: { type: "array", items: { type: "string" } },
            getterCalleePatterns: { type: "array", items: { type: "string" } },
            requiredWrapperCallee: { type: "string" },
            nullableReturnTypeNames: { type: "array", items: { type: "string" } },
            inferNullableFromTopLevelEntityPath: { type: "boolean" },
            topLevelEntityPathPatterns: { type: "array", items: { type: "string" } },
            unanalyzable: { enum: ["finding", "ignore"] },
          },
          required: ["getterCalleePatterns", "requiredWrapperCallee"],
          additionalProperties: false,
        },
      ],
      messages: {
        wrapper:
          "Wrap nullable entity fetch '{{callee}}' in {{wrapper}} so missing entities map to null consistently.",
        unanalyzable:
          "Could not resolve whether '{{callee}}' is the configured getter or wrapper. Use a static import.",
      },
    },
    (context) => {
      const options = context.options?.[0] ?? {};
      if (!pathAllowed(context.filename, options)) return {};
      const suppressed = ruleSuppression(context, "server-require-nullable-fetch-wrapper");
      const getterPatterns = compilePatterns(options.getterCalleePatterns);
      const nullableNames = new Set(options.nullableReturnTypeNames ?? []);
      const exportedNames = new Set();
      const functionTypes = new Map();
      const overloadReturnTypes = new Map();
      const functionStack = [];

      function checkedFunction(node) {
        const name = functionName(node);
        if (!isExportedFunction(node) && (!name || !exportedNames.has(name))) return false;
        const returnType =
          functionReturnAnnotation(node, functionTypes) || overloadReturnTypes.get(name);
        return (
          typeMatchesNullableHint(returnType, nullableNames) ||
          hasCheckedPath(context.filename, options)
        );
      }

      function enterFunction(node) {
        functionStack.push(currentFunctionChecked() || checkedFunction(node));
      }

      function exitFunction() {
        functionStack.pop();
      }

      function currentFunctionChecked() {
        return functionStack[functionStack.length - 1] === true;
      }

      return {
        Program(node) {
          for (const name of collectExportedNames(node)) exportedNames.add(name);
          for (const [name, returnType] of collectFunctionTypeReturns(node)) {
            functionTypes.set(name, returnType);
          }
          for (const [name, returnType] of collectFunctionOverloadReturnTypes(
            node,
            functionTypes,
          )) {
            overloadReturnTypes.set(name, returnType);
          }
        },
        FunctionDeclaration: enterFunction,
        "FunctionDeclaration:exit": exitFunction,
        FunctionExpression: enterFunction,
        "FunctionExpression:exit": exitFunction,
        ArrowFunctionExpression: enterFunction,
        "ArrowFunctionExpression:exit": exitFunction,
        CallExpression(node) {
          if (!currentFunctionChecked()) return;
          const invoked = logicalCall(node);
          const direct = calleeNames(node.callee, context);
          const forwarded = invoked ? calleeNames(invoked.callee, context) : null;
          const names = [...direct.names, ...(forwarded?.names ?? [])];
          const unresolved = direct.unresolved || forwarded?.unresolved === true;
          const name = names.find((candidate) =>
            getterPatterns.some((pattern) => pattern.test(candidate)),
          );
          const wrapped = wrapperState(node, options.requiredWrapperCallee, context);
          if (wrapped === "inside") return;
          if (suppressed(node)) return;
          const member = memberPropertyName(invoked ? invoked.callee : node.callee);
          const knownOtherMember =
            Boolean(member) && !getterPatterns.some((pattern) => pattern.test(member));
          const uncertainGetter =
            (Boolean(name) && (wrapped === "unresolved" || unresolved)) ||
            (!name && unresolved && !knownOtherMember);
          if (uncertainGetter) {
            if (reportsUnanalyzable(context)) {
              context.report({
                node,
                messageId: "unanalyzable",
                data: { callee: name || "this call" },
              });
            }
            return;
          }
          if (name) {
            context.report({
              node,
              messageId: "wrapper",
              data: { callee: name, wrapper: options.requiredWrapperCallee },
            });
          }
        },
      };
    },
  ),
  {
    __test: {
      ...helpers,
      insideWrapper,
    },
  },
);
