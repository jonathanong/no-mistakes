"use strict";

const { typeAnnotation, typeName } = require("../react-node-types");
const {
  memberPropertyName,
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
  let current = node;
  if (!current) return null;
  if (current.type === "ChainExpression") current = current.expression;
  if (current.type === "Identifier") return current.name;
  if (current.type !== "MemberExpression") return null;
  const object = calleePath(current.object);
  const property = memberPropertyName(current);
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
  const current = unwrapTransparentReturnType(node);
  if (!current) return false;
  if (current.type === "TSNullKeyword") return true;
  const name = typeName(current);
  if (name && names.has(name)) return true;
  if (current.type === "TSUnionType") {
    return current.types.some((item) => typeMatchesNullableHint(item, names));
  }
  return false;
}

function typeArguments(node) {
  return node.typeArguments?.params || node.typeParameters?.params || [];
}

function unwrapTransparentReturnType(node) {
  const current = unwrapType(node);
  if (typeName(current) !== "Promise") return current;
  const [value] = typeArguments(current);
  return value ? unwrapTransparentReturnType(value) : current;
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

function functionName(node) {
  if (node.type === "FunctionDeclaration") return node.id?.name || null;
  const declarator = variableDeclarator(node);
  return declarator?.id?.type === "Identifier" ? declarator.id.name : null;
}

function functionTypeReturn(type, functionTypes = new Map()) {
  const current = unwrapType(type);
  if (current?.type === "TSFunctionType") return returnTypeAnnotation(current);
  const name = typeName(current);
  if (name) return functionTypes.get(name) || null;
  return null;
}

function functionReturnAnnotation(node, functionTypes = new Map()) {
  const direct = returnTypeAnnotation(node);
  if (direct) return direct;
  const declarator = variableDeclarator(node);
  return declarator?.id ? functionTypeReturn(typeAnnotation(declarator.id), functionTypes) : null;
}

function hasCheckedPath(filename, options) {
  return (
    options.inferNullableFromTopLevelEntityPath === true &&
    stringMatches(repoRelativeFilename(filename), options.topLevelEntityPathPatterns ?? [])
  );
}

function collectExportedNames(program) {
  const names = new Set();
  for (const statement of program.body || []) {
    if (
      statement.type === "ExportDefaultDeclaration" &&
      statement.declaration.type === "Identifier"
    ) {
      names.add(statement.declaration.name);
    }
    if (statement.type !== "ExportNamedDeclaration" || statement.source) continue;
    for (const specifier of statement.specifiers || []) {
      if (specifier.type === "ExportSpecifier" && specifier.local.type === "Identifier") {
        names.add(specifier.local.name);
      }
    }
  }
  return names;
}

function declarationOf(statement) {
  return statement.type === "ExportNamedDeclaration" && statement.declaration
    ? statement.declaration
    : statement;
}

function collectFunctionTypeReturns(program) {
  const types = new Map();
  for (const statement of program.body || []) {
    const declaration = declarationOf(statement);
    if (declaration.type !== "TSTypeAliasDeclaration") continue;
    const returnType = functionTypeReturn(declaration.typeAnnotation, types);
    if (returnType) types.set(declaration.id.name, returnType);
  }
  return types;
}

function collectFunctionOverloadReturnTypes(program, functionTypes = new Map()) {
  const types = new Map();
  for (const statement of program.body || []) {
    const declaration = declarationOf(statement);
    if (
      !["FunctionDeclaration", "TSDeclareFunction"].includes(declaration.type) ||
      declaration.body ||
      declaration.id?.type !== "Identifier"
    ) {
      continue;
    }
    const returnType = functionReturnAnnotation(declaration, functionTypes);
    if (returnType) types.set(declaration.id.name, returnType);
  }
  return types;
}

module.exports = {
  calleePath,
  collectExportedNames,
  collectFunctionTypeReturns,
  collectFunctionOverloadReturnTypes,
  compilePatterns,
  functionName,
  functionReturnAnnotation,
  functionTypeReturn,
  hasCheckedPath,
  isExportedFunction,
  typeMatchesNullableHint,
};
