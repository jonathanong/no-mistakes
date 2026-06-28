"use strict";

const { rule } = require("../helpers");
const { typeAnnotation, typeName } = require("../react-node-types");
const {
  memberPropertyName,
  pathAllowed,
  repoRelativeFilename,
  stringMatches,
} = require("./module-mock-helpers");

function compilePatterns(patterns = []) {
  return patterns.flatMap((pattern) => {
    try {
      return [new RegExp(pattern)];
    } catch {
      return [];
    }
  });
}

function calleePath(node) {
  if (!node) return null;
  if (node.type === "Identifier") return node.name;
  if (node.type !== "MemberExpression") return null;
  const object = calleePath(node.object);
  const property = memberPropertyName(node);
  return object && property ? `${object}.${property}` : null;
}

function unwrapType(node) {
  let current = node;
  while (
    current &&
    (current.type === "TSParenthesizedType" ||
      current.type === "TSOptionalType" ||
      current.type === "TSRestType")
  ) {
    current = current.typeAnnotation;
  }
  return current;
}

function typeMatchesNullableHint(node, names) {
  const current = unwrapType(node);
  if (!current) return false;
  if (current.type === "TSNullKeyword") return true;
  const name = typeName(current);
  if (name && names.has(name)) return true;
  if (current.type === "TSUnionType") {
    return current.types.some((item) => typeMatchesNullableHint(item, names));
  }
  if (current.type === "TSTypeReference") {
    return (current.typeArguments?.params || current.typeParameters?.params || []).some((item) =>
      typeMatchesNullableHint(item, names),
    );
  }
  return false;
}

function functionTypeReturn(type) {
  const current = unwrapType(type);
  if (current?.type === "TSFunctionType") return returnTypeAnnotation(current);
  return null;
}

function returnTypeAnnotation(node) {
  return node?.returnType?.type === "TSTypeAnnotation"
    ? unwrapType(node.returnType.typeAnnotation)
    : null;
}

function variableDeclarator(node) {
  return node.parent?.type === "VariableDeclarator" ? node.parent : null;
}

function variableDeclaration(node) {
  const declarator = variableDeclarator(node);
  return declarator?.parent?.type === "VariableDeclaration" ? declarator.parent : null;
}

function isExportedFunction(node) {
  if (node.parent?.type === "ExportDefaultDeclaration") return true;
  if (node.type === "FunctionDeclaration") {
    return node.parent?.type === "ExportNamedDeclaration";
  }
  const declaration = variableDeclaration(node);
  return declaration?.parent?.type === "ExportNamedDeclaration";
}

function functionReturnAnnotation(node) {
  const direct = returnTypeAnnotation(node);
  if (direct) return direct;
  const declarator = variableDeclarator(node);
  return declarator?.id ? functionTypeReturn(typeAnnotation(declarator.id)) : null;
}

function hasCheckedPath(filename, options) {
  return (
    options.inferNullableFromTopLevelEntityPath === true &&
    stringMatches(repoRelativeFilename(filename), options.topLevelEntityPathPatterns ?? [])
  );
}

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
      const functionStack = [];

      function checkedFunction(node) {
        if (!isExportedFunction(node)) return false;
        const returnType = functionReturnAnnotation(node);
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
      calleePath,
      compilePatterns,
      functionReturnAnnotation,
      functionTypeReturn,
      insideWrapper,
      isExportedFunction,
      typeMatchesNullableHint,
    },
  },
);
