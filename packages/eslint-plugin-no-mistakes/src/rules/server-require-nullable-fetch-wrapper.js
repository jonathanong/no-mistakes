"use strict";

const { rule } = require("../helpers");
const { pathAllowed } = require("./module-mock-helpers");
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

function insideWrapper(node, wrapper) {
  let current = node.parent;
  while (current) {
    if (current.type === "CallExpression" && calleePath(current.callee) === wrapper) return true;
    if (isFunctionBoundary(current)) return false;
    current = current.parent;
  }
  return false;
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
          },
          required: ["getterCalleePatterns", "requiredWrapperCallee"],
          additionalProperties: false,
        },
      ],
      messages: {
        wrapper:
          "Wrap nullable entity fetch '{{callee}}' in {{wrapper}} so missing entities map to null consistently.",
      },
    },
    (context) => {
      const options = context.options?.[0] ?? {};
      if (!pathAllowed(context.filename, options)) return {};
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
        functionStack.push(checkedFunction(node));
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
          const name = calleePath(node.callee);
          if (!name || !getterPatterns.some((pattern) => pattern.test(name))) return;
          if (insideWrapper(node, options.requiredWrapperCallee)) return;
          context.report({
            node,
            messageId: "wrapper",
            data: { callee: name, wrapper: options.requiredWrapperCallee },
          });
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
