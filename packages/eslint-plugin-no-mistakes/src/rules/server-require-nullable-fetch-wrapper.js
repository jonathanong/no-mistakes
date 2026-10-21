"use strict";

const { rule } = require("../helpers");
const { logicalCall, pathAllowed } = require("./module-mock-helpers");
const { resolveVariable } = require("./async-target-bindings");
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

function rootIdentifier(node) {
  let current = node;
  while (current?.type === "ChainExpression" || current?.type === "MemberExpression") {
    current = current.type === "ChainExpression" ? current.expression : current.object;
  }
  return current?.type === "Identifier" ? current : null;
}

function spelledCallee(callee, context) {
  const path = calleePath(callee);
  if (!path) return null;
  const root = rootIdentifier(callee);
  if (!root) return path;
  const variable = resolveVariable(root, context);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  if (def?.type !== "ImportBinding" || def.node?.type !== "ImportSpecifier") return path;
  const imported = def.node.imported;
  const importedName = imported?.type === "Literal" ? String(imported.value) : imported?.name;
  if (!importedName || importedName === root.name) return path;
  return `${importedName}${path.slice(root.name.length)}`;
}

function wrapperMatches(callee, wrapper, context) {
  return calleePath(callee) === wrapper || spelledCallee(callee, context) === wrapper;
}

function insideWrapper(node, wrapper, context) {
  let current = node.parent;
  while (current) {
    const invoked = current.type === "CallExpression" ? logicalCall(current) : null;
    if (invoked && wrapperMatches(invoked.callee, wrapper, context)) return true;
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
          const name = invoked ? spelledCallee(invoked.callee, context) : null;
          if (!name || !getterPatterns.some((pattern) => pattern.test(name))) return;
          if (insideWrapper(node, options.requiredWrapperCallee, context)) return;
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
