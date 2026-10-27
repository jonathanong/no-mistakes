"use strict";
const { typeArguments } = require("./async-source-ts-expressions");
const { compositeType } = require("./async-source-ts-types");

function declaration(node, api) {
  const { ts, convert: c, make: m, list, fn, annotation, modifiers, property } = api;
  const kind = ts.SyntaxKind;
  const composite = compositeType(node, api);
  if (composite) return composite;
  if (
    ts.isFunctionDeclaration(node) ||
    ts.isVariableStatement(node) ||
    ts.isInterfaceDeclaration(node) ||
    ts.isTypeAliasDeclaration(node)
  ) {
    const has = modifiers(node);
    let value;
    if (ts.isFunctionDeclaration(node))
      value = fn(node, node.body ? "FunctionDeclaration" : "TSDeclareFunction");
    if (ts.isVariableStatement(node)) value = c(node.declarationList);
    if (ts.isTypeAliasDeclaration(node))
      value = m(node, "TSTypeAliasDeclaration", {
        id: c(node.name),
        typeAnnotation: c(node.type),
        typeParameters: parameters(node, api),
      });
    if (ts.isInterfaceDeclaration(node))
      value = m(node, "TSInterfaceDeclaration", {
        id: c(node.name),
        body: m(node, "TSInterfaceBody", { body: list(node.members) }),
        extends: node.heritageClauses?.flatMap((item) => list(item.types)) || [],
        typeParameters: parameters(node, api),
      });
    if (has(kind.DefaultKeyword))
      return m(node, "ExportDefaultDeclaration", { declaration: value });
    return has(kind.ExportKeyword)
      ? m(node, "ExportNamedDeclaration", {
          declaration: value,
          specifiers: [],
          source: null,
          exportKind:
            ts.isInterfaceDeclaration(node) || ts.isTypeAliasDeclaration(node) ? "type" : "value",
        })
      : value;
  }
  if (ts.isVariableDeclarationList(node)) {
    if (node.flags & (ts.NodeFlags.Using || 0)) throw new Error("Unsupported resource binding");
    return m(node, "VariableDeclaration", {
      kind:
        node.flags & ts.NodeFlags.Const ? "const" : node.flags & ts.NodeFlags.Let ? "let" : "var",
      declarations: list(node.declarations),
    });
  }
  if (ts.isVariableDeclaration(node)) {
    const id = c(node.name);
    if (node.type) id.typeAnnotation = annotation(node.type);
    // Identifier visitor keys must also include annotations added after creation.
    if (id.typeAnnotation) m(node.name, id.type, { typeAnnotation: id.typeAnnotation });
    return m(node, "VariableDeclarator", { id, init: c(node.initializer) });
  }
  if (ts.isParameter(node) || ts.isBindingElement(node)) {
    modifiers(node);
    // TypeScript's erased `this` parameter does not occupy a runtime argument slot.
    if (ts.isParameter(node) && ts.isIdentifier(node.name) && node.name.text === "this")
      throw new Error("Unsupported this parameter");
    let value = c(node.name);
    if (node.type) {
      value.typeAnnotation = annotation(node.type);
      m(node.name, value.type, { typeAnnotation: value.typeAnnotation });
    }
    if (node.initializer)
      value = m(node, "AssignmentPattern", { left: value, right: c(node.initializer) });
    if (node.dotDotDotToken) value = m(node, "RestElement", { argument: value });
    if (
      ts.isBindingElement(node) &&
      ts.isObjectBindingPattern(node.parent) &&
      !node.dotDotDotToken
    ) {
      const name = node.propertyName || node.name;
      const computed = ts.isComputedPropertyName(name);
      return m(node, "Property", {
        key: c(computed ? name.expression : name),
        computed,
        value,
        method: false,
        kind: "init",
        shorthand: !node.propertyName,
      });
    }
    return value;
  }
  if (ts.isObjectBindingPattern(node))
    return m(node, "ObjectPattern", { properties: list(node.elements) });
  if (ts.isArrayBindingPattern(node))
    return m(node, "ArrayPattern", {
      elements: Array.from(node.elements, (item) =>
        ts.isOmittedExpression(item) ? null : c(item),
      ),
    });
  if (ts.isTypeReferenceNode(node))
    return m(node, "TSTypeReference", {
      typeName: c(node.typeName),
      typeArguments: typeArguments(node, api),
    });
  if (ts.isTypeQueryNode(node))
    return m(node, "TSTypeQuery", {
      exprName: c(node.exprName),
      typeArguments: typeArguments(node, api),
    });
  if (ts.isQualifiedName(node))
    return m(node, "TSQualifiedName", { left: c(node.left), right: c(node.right) });
  if (ts.isTypeLiteralNode(node)) return m(node, "TSTypeLiteral", { members: list(node.members) });
  if (ts.isPropertySignature(node))
    return m(node, "TSPropertySignature", {
      ...property(node),
      optional: Boolean(node.questionToken),
      typeAnnotation: annotation(node.type),
    });
  if (ts.isMethodSignature(node))
    return m(node, "TSMethodSignature", {
      ...property(node),
      params: list(node.parameters),
      returnType: annotation(node.type),
      typeParameters: parameters(node, api),
      optional: Boolean(node.questionToken),
    });
  if (ts.isIndexSignatureDeclaration(node))
    return m(node, "TSIndexSignature", {
      parameters: list(node.parameters),
      typeAnnotation: annotation(node.type),
    });
  if (ts.isFunctionTypeNode(node))
    return m(node, "TSFunctionType", {
      params: list(node.parameters),
      returnType: annotation(node.type),
      typeParameters: parameters(node, api),
    });
  if (ts.isTypeParameterDeclaration(node))
    return m(node, "TSTypeParameter", {
      name: c(node.name),
      constraint: c(node.constraint),
      default: c(node.default),
    });
  if (ts.isUnionTypeNode(node) || ts.isIntersectionTypeNode(node))
    return m(node, ts.isUnionTypeNode(node) ? "TSUnionType" : "TSIntersectionType", {
      types: list(node.types),
    });
  if (ts.isArrayTypeNode(node)) return m(node, "TSArrayType", { elementType: c(node.elementType) });
  if (ts.isLiteralTypeNode(node)) return m(node, "TSLiteralType", { literal: c(node.literal) });
  if (ts.isTypeOperatorNode(node))
    return m(node, "TSTypeOperator", {
      operator: ts.tokenToString(node.operator),
      typeAnnotation: c(node.type),
    });
  if (ts.isParenthesizedTypeNode(node))
    return m(node, "TSParenthesizedType", { typeAnnotation: c(node.type) });
  if (ts.isExpressionWithTypeArguments(node))
    return m(node, "TSInterfaceHeritage", {
      expression: c(node.expression),
      typeArguments: typeArguments(node, api),
    });
  const keywords = new Map([
    [kind.AnyKeyword, "TSAnyKeyword"],
    [kind.UnknownKeyword, "TSUnknownKeyword"],
    [kind.StringKeyword, "TSStringKeyword"],
    [kind.NumberKeyword, "TSNumberKeyword"],
    [kind.BooleanKeyword, "TSBooleanKeyword"],
    [kind.VoidKeyword, "TSVoidKeyword"],
    [kind.NeverKeyword, "TSNeverKeyword"],
    [kind.UndefinedKeyword, "TSUndefinedKeyword"],
    [kind.ObjectKeyword, "TSObjectKeyword"],
    [kind.SymbolKeyword, "TSSymbolKeyword"],
  ]);
  return keywords.has(node.kind) ? m(node, keywords.get(node.kind)) : null;
}

function parameters(node, { make, list }) {
  return (
    node.typeParameters &&
    make(node, "TSTypeParameterDeclaration", { params: list(node.typeParameters) })
  );
}

module.exports = { declaration };
