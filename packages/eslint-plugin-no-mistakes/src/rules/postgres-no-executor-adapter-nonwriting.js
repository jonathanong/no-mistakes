"use strict";

const { safeOtherArguments } = require("./postgres-no-executor-adapter-nonwriting-arguments");

const { pureReaderBody } = require("./postgres-no-executor-adapter-nonwriting-body");
const { sourceWrites } = require("./postgres-no-executor-adapter-source");
const { primitiveType } = require("./postgres-no-executor-adapter-sql-primitive");

function directTarget(call, checker, ts, facts, program) {
  const callee = call.expression;
  if (
    call.questionDotToken ||
    !ts.isIdentifier(callee) ||
    call.arguments.some((argument) => ts.isSpreadElement(argument))
  )
    return null;
  const type = checker.getTypeAtLocation(callee);
  if (type.flags & (ts.TypeFlags.Any | ts.TypeFlags.Unknown | ts.TypeFlags.Union)) return null;
  if (checker.getSignaturesOfType(type, ts.SignatureKind.Call).length !== 1) return null;

  const declaration = checker.getResolvedSignature(call)?.getDeclaration();
  if (
    !declaration?.body ||
    !ts.isFunctionDeclaration(declaration) ||
    declaration.asteriskToken ||
    declaration.modifiers?.some((modifier) => modifier.kind === ts.SyntaxKind.AsyncKeyword)
  )
    return null;
  const symbol = checker.getSymbolAtLocation(callee);
  const target = symbol?.flags & ts.SymbolFlags.Alias ? checker.getAliasedSymbol(symbol) : symbol;
  if (target?.declarations?.length !== 1 || target.declarations[0] !== declaration) return null;
  const callerWrites = sourceWrites(call.getSourceFile(), checker, facts, program);
  const targetWrites = sourceWrites(declaration.getSourceFile(), checker, facts, program);
  return callerWrites.eval ||
    targetWrites.eval ||
    callerWrites.symbols.has(symbol) ||
    targetWrites.symbols.has(target)
    ? null
    : declaration;
}

function primitiveFields(parameter, checker, ts) {
  const type = checker.getTypeAtLocation(parameter.name);
  if (
    !(type.flags & ts.TypeFlags.Object) ||
    checker.getSignaturesOfType(type, ts.SignatureKind.Call).length ||
    checker.getSignaturesOfType(type, ts.SignatureKind.Construct).length ||
    checker.getIndexInfosOfType(type).length
  )
    return null;
  const properties = checker.getPropertiesOfType(type);
  if (!properties.length) return null;
  const fields = new Set();
  for (const property of properties) {
    const declaration = property.declarations?.[0];
    if (
      property.flags & ts.SymbolFlags.Optional ||
      property.declarations?.length !== 1 ||
      !(ts.isPropertySignature(declaration) || ts.isPropertyDeclaration(declaration)) ||
      !primitiveType(checker.getTypeOfSymbolAtLocation(property, parameter), ts, false)
    )
      return null;
    fields.add(property.getName());
  }
  return fields;
}

function nonWritingReader(declaration, index, checker, ts, facts, program) {
  if (
    declaration.parameters.some(
      (parameter) =>
        !ts.isIdentifier(parameter.name) ||
        parameter.name.text === "this" ||
        parameter.initializer ||
        parameter.dotDotDotToken ||
        parameter.questionToken ||
        parameter.modifiers?.length,
    )
  )
    return false;
  const parameter = declaration.parameters[index];
  if (!parameter) return false;
  const symbol = checker.getSymbolAtLocation(parameter.name);
  if (
    !symbol ||
    sourceWrites(declaration.getSourceFile(), checker, facts, program).symbols.has(symbol)
  )
    return false;
  const fields = primitiveFields(parameter, checker, ts);
  return Boolean(fields && pureReaderBody(declaration, symbol, fields, checker, ts));
}

function tsNonWritingArgument(call, index, checker, ts, facts, program) {
  let argument = call.arguments[index];
  while (ts.isParenthesizedExpression(argument)) argument = argument.expression;
  if (!ts.isIdentifier(argument)) return false;
  const declaration = directTarget(call, checker, ts, facts, program);
  if (!declaration || !safeOtherArguments(call, index, checker, ts, facts, program)) return false;
  const readers = (facts.nonWritingReaders ??= new WeakMap());
  let byIndex = readers.get(declaration);
  if (!byIndex) {
    byIndex = new Map();
    readers.set(declaration, byIndex);
  }
  if (!byIndex.has(index))
    byIndex.set(index, nonWritingReader(declaration, index, checker, ts, facts, program));
  return byIndex.get(index);
}

module.exports = { tsNonWritingArgument };
