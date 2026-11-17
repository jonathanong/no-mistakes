"use strict";

const { childNodes, resolveVariable, unwrapTs } = require("./postgres-query-text");
const { sourceWrites } = require("./postgres-no-executor-adapter-source");

function primitiveAnnotation(node, allowUndefined) {
  if (node?.type === "TSUnionType") {
    return (
      node.types.length > 0 && node.types.every((part) => primitiveAnnotation(part, allowUndefined))
    );
  }
  if (node?.type === "TSUndefinedKeyword") return allowUndefined;
  if (node?.type === "TSLiteralType") {
    return (
      node.literal?.type === "Literal" &&
      (node.literal.value === null ||
        ["string", "number", "boolean", "bigint"].includes(typeof node.literal.value))
    );
  }
  return [
    "TSStringKeyword",
    "TSNumberKeyword",
    "TSBooleanKeyword",
    "TSBigIntKeyword",
    "TSNullKeyword",
  ].includes(node?.type);
}

function primitiveType(type, ts, allowUndefined) {
  if (type.isUnion())
    return (
      type.types.length > 0 && type.types.every((part) => primitiveType(part, ts, allowUndefined))
    );
  const mask =
    ts.TypeFlags.StringLike |
    ts.TypeFlags.NumberLike |
    ts.TypeFlags.BooleanLike |
    ts.TypeFlags.BigIntLike |
    ts.TypeFlags.Null |
    (allowUndefined ? ts.TypeFlags.Undefined : 0);
  return Boolean(type.flags & mask);
}

function scalarArgument(node, context, allowUndefined = true, seen = new Set()) {
  node = unwrapTs(node);
  if (node?.type === "Literal")
    return (
      node.value === null || ["string", "number", "boolean", "bigint"].includes(typeof node.value)
    );
  if (node?.type === "TemplateLiteral") return node.expressions.length === 0;
  if (node?.type === "TaggedTemplateExpression") return false;
  if (node?.type === "Identifier") {
    const variable = resolveVariable(node, context);
    if (variable && !seen.has(variable)) {
      seen.add(variable);
      if (variable.references.some((reference) => reference.isWrite() && !reference.init))
        return false;
      for (const def of variable.defs) {
        if (def.type === "Variable") {
          return (
            Boolean(def.node.init) && scalarArgument(def.node.init, context, allowUndefined, seen)
          );
        }
        if (
          def.type === "Parameter" &&
          primitiveAnnotation(def.name?.typeAnnotation?.typeAnnotation, allowUndefined)
        )
          return true;
      }
      if (variable.defs.some((def) => def.type !== "Parameter")) return false;
    }
  }
  const parameter = node?.type === "Identifier" ? resolveVariable(node, context) : null;
  if (!parameter?.defs.length || !parameter.defs.every((def) => def.type === "Parameter"))
    return false;
  const services = context.sourceCode.parserServices;
  if (!services?.program || !services.esTreeNodeToTSNodeMap?.has(node)) return false;
  const ts = require("typescript");
  const checker = services.program.getTypeChecker();
  return primitiveType(
    checker.getTypeAtLocation(services.esTreeNodeToTSNodeMap.get(node)),
    ts,
    allowUndefined,
  );
}

function localHoleSafe(expression, call, fn, context) {
  const parameters = fn.params.map((param) => {
    const target = param.type === "AssignmentPattern" ? param.left : param;
    return target.type === "Identifier" ? resolveVariable(target, context) : null;
  });
  const indexes = new Set();
  const seen = new Set();
  function visit(node) {
    if (node.type === "MemberExpression" && !node.computed) return visit(node.object);
    if (node.type === "Identifier") {
      const variable = resolveVariable(node, context);
      if (!variable || seen.has(variable)) return;
      seen.add(variable);
      const index = parameters.indexOf(variable);
      if (index >= 0) {
        indexes.add(index);
        if (variable.references.some((ref) => ref.isWrite() && !ref.init)) indexes.add(-1);
      } else if (variable.defs.some((def) => def.type === "Parameter")) indexes.add(-1);
      else
        for (const def of variable.defs) {
          if (def.type === "Variable") {
            if (variable.references.some((ref) => ref.isWrite() && !ref.init)) indexes.add(-1);
            if (def.node.init) visit(def.node.init);
          }
        }
      return;
    }
    for (const child of childNodes(node)) visit(child);
  }
  visit(expression);
  return [...indexes].every((index) => {
    if (index < 0) return false;
    const fallback = fn.params[index]?.type === "AssignmentPattern" ? fn.params[index].right : null;
    const defaultScalar = fallback ? scalarArgument(fallback, context) : true;
    return call.arguments[index]
      ? scalarArgument(call.arguments[index], context, defaultScalar)
      : Boolean(fallback && defaultScalar);
  });
}

function tsScalarDefault(node, ts) {
  return Boolean(
    node &&
    (ts.isStringLiteral(node) ||
      ts.isNumericLiteral(node) ||
      ts.isNoSubstitutionTemplateLiteral(node) ||
      [ts.SyntaxKind.TrueKeyword, ts.SyntaxKind.FalseKeyword, ts.SyntaxKind.NullKeyword].includes(
        node.kind,
      )),
  );
}

function tsHoleSafe(expression, call, declaration, checker, ts, context, facts, program) {
  const parameters = declaration.parameters.map((param) =>
    ts.isIdentifier(param.name) ? checker.getSymbolAtLocation(param.name) : null,
  );
  const indexes = new Set();
  const seen = new Set();
  function visit(node) {
    if (ts.isPropertyAccessExpression(node)) return visit(node.expression);
    if (ts.isIdentifier(node)) {
      const symbol = checker.getSymbolAtLocation(node);
      if (!symbol || seen.has(symbol)) return;
      seen.add(symbol);
      const index = parameters.indexOf(symbol);
      if (index >= 0) {
        indexes.add(index);
        if (sourceWrites(declaration.getSourceFile(), checker, facts, program).symbols.has(symbol))
          indexes.add(-1);
      } else
        for (const source of symbol.declarations ?? []) {
          if (ts.isParameter(source) || ts.isBindingElement(source)) indexes.add(-1);
          if (ts.isVariableDeclaration(source)) {
            if (
              sourceWrites(declaration.getSourceFile(), checker, facts, program).symbols.has(symbol)
            )
              indexes.add(-1);
            if (source.initializer) visit(source.initializer);
          }
        }
      return;
    }
    ts.forEachChild(node, visit);
  }
  visit(expression);
  return [...indexes].every((index) => {
    if (index < 0) return false;
    const fallback = declaration.parameters[index]?.initializer;
    const defaultScalar = fallback ? tsScalarDefault(fallback, ts) : true;
    return call.arguments[index]
      ? scalarArgument(call.arguments[index], context, defaultScalar)
      : Boolean(fallback && defaultScalar);
  });
}

module.exports = { localHoleSafe, tsHoleSafe };
