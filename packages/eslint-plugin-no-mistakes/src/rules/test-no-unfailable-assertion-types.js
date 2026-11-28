"use strict";

const { unwrap } = require("./test-message-expect");

const PLAYWRIGHT_PATH = /(?:^|[\\/])(?:@playwright|playwright(?:-core)?)(?:[\\/]|$)/;
const PLAYWRIGHT_MODULES = new Set(["@playwright/test", "playwright", "playwright-core"]);
const PLAYWRIGHT_RECEIVERS = new Set(["Page", "Frame", "FrameLocator", "Locator"]);
const PLAYWRIGHT_LOCATORS = new Set(["Locator", "FrameLocator"]);
const BOXED_WRAPPERS = new Set(["Object", "String", "Number", "Boolean", "BigInt"]);

function getTypeContext(context) {
  const services = context.sourceCode?.parserServices;
  if (!services?.program || !services.esTreeNodeToTSNodeMap) return null;
  const compilerOptions = services.program.getCompilerOptions();
  if (compilerOptions.noResolve) return null;
  return {
    program: services.program,
    checker: services.program.getTypeChecker(),
    tsNodeMap: services.esTreeNodeToTSNodeMap,
    ts: require("typescript"),
    strictNullChecks: compilerOptions.strictNullChecks ?? Boolean(compilerOptions.strict),
    noUncheckedIndexedAccess: Boolean(compilerOptions.noUncheckedIndexedAccess),
  };
}

function getExpressionType(node, typeCtx, awaited) {
  const tsNode = typeCtx.tsNodeMap.get(node);
  if (!tsNode) return null;
  const type = typeCtx.checker.getTypeAtLocation(tsNode);
  return awaited ? typeCtx.checker.getAwaitedType(type) : type;
}

function hasIndexableType(type, checker) {
  if (type.isUnion()) return type.types.some((part) => hasIndexableType(part, checker));
  return (
    checker.isArrayType(type) ||
    checker.isTupleType(type) ||
    checker.getIndexInfosOfType(type).length > 0
  );
}

function isUncheckedIndexedAccess(node, typeCtx) {
  if (typeCtx.noUncheckedIndexedAccess) return false;
  const current = unwrap(node);
  if (current.type !== "MemberExpression" || !current.computed) return false;
  const objectType = getExpressionType(current.object, typeCtx, false);
  return Boolean(objectType && hasIndexableType(objectType, typeCtx.checker));
}

function resolveConstrainedType(type, checker, ts) {
  const genericFlags =
    ts.TypeFlags.TypeParameter |
    ts.TypeFlags.Index |
    ts.TypeFlags.IndexedAccess |
    ts.TypeFlags.Conditional |
    ts.TypeFlags.Substitution;
  return type.flags & genericFlags ? (checker.getBaseConstraintOfType(type) ?? null) : type;
}

function excludesUndefined(type, checker, ts) {
  const resolved = resolveConstrainedType(type, checker, ts);
  if (!resolved) return false;
  if (
    resolved.flags &
    (ts.TypeFlags.Any |
      ts.TypeFlags.Unknown |
      ts.TypeFlags.Never |
      ts.TypeFlags.Undefined |
      ts.TypeFlags.Void)
  ) {
    return false;
  }
  if (resolved.isUnion()) {
    return resolved.types.every((part) => excludesUndefined(part, checker, ts));
  }
  if (resolved.isIntersection()) {
    return resolved.types.some((part) => excludesUndefined(part, checker, ts));
  }
  return true;
}

function isAlwaysTruthyObject(type, checker, ts) {
  const symbolName = type.symbol?.name;
  if (symbolName && BOXED_WRAPPERS.has(symbolName)) return false;
  if (
    checker.getSignaturesOfType(type, ts.SignatureKind.Call).length > 0 ||
    checker.getSignaturesOfType(type, ts.SignatureKind.Construct).length > 0 ||
    checker.isArrayType(type) ||
    checker.isTupleType(type) ||
    checker.getIndexInfosOfType(type).length > 0
  ) {
    return true;
  }
  return type.getProperties().length > 0;
}

function isPossiblyFalsyPrimitive(type, ts) {
  return Boolean(
    type.flags &
    (ts.TypeFlags.Any |
      ts.TypeFlags.Unknown |
      ts.TypeFlags.Never |
      ts.TypeFlags.Undefined |
      ts.TypeFlags.Void |
      ts.TypeFlags.Null |
      ts.TypeFlags.String |
      ts.TypeFlags.Number |
      ts.TypeFlags.Boolean |
      ts.TypeFlags.BigInt |
      ts.TypeFlags.Enum),
  );
}

function isAlwaysTruthy(type, checker, ts) {
  const resolved = resolveConstrainedType(type, checker, ts);
  if (!resolved || isPossiblyFalsyPrimitive(resolved, ts)) return false;
  if (resolved.isUnion()) {
    return resolved.types.every((part) => isAlwaysTruthy(part, checker, ts));
  }
  if (resolved.isIntersection()) {
    return (
      !resolved.types.some((part) => isPossiblyFalsyPrimitive(part, ts)) &&
      resolved.types.some((part) => isAlwaysTruthy(part, checker, ts))
    );
  }
  if (resolved.flags & ts.TypeFlags.StringLiteral) return resolved.value !== "";
  if (resolved.flags & ts.TypeFlags.NumberLiteral) return resolved.value !== 0;
  if (resolved.flags & ts.TypeFlags.BigIntLiteral) return resolved.value.base10Value !== "0";
  if (resolved.flags & ts.TypeFlags.BooleanLiteral)
    return checker.typeToString(resolved) === "true";
  if (resolved.flags & (ts.TypeFlags.ESSymbolLike | ts.TypeFlags.NonPrimitive)) return true;
  return Boolean(
    resolved.flags & ts.TypeFlags.Object && isAlwaysTruthyObject(resolved, checker, ts),
  );
}

function isPlaywrightDeclaration(declaration, ts) {
  if (PLAYWRIGHT_PATH.test(declaration.getSourceFile().fileName)) return true;
  for (let current = declaration.parent; current; current = current.parent) {
    if (ts.isModuleDeclaration(current) && PLAYWRIGHT_MODULES.has(current.name.text)) return true;
  }
  return false;
}

function isPlaywrightType(type, allowedNames, ts) {
  const symbol = type.symbol ?? type.aliasSymbol;
  if (!symbol || !allowedNames.has(symbol.name)) return false;
  return Boolean(symbol.declarations?.some((decl) => isPlaywrightDeclaration(decl, ts)));
}

function unwrapDiscardedCall(expression) {
  let current = expression;
  while (
    current?.type === "ChainExpression" ||
    current?.type === "TSAsExpression" ||
    current?.type === "TSTypeAssertion" ||
    current?.type === "TSNonNullExpression" ||
    current?.type === "TSSatisfiesExpression" ||
    (current?.type === "UnaryExpression" && current.operator === "void")
  ) {
    current = current.type === "UnaryExpression" ? current.argument : current.expression;
  }
  return current?.type === "CallExpression" && current.callee.type === "MemberExpression"
    ? current
    : null;
}

function isDiscardedPlaywrightLocator(statement, typeCtx) {
  const call = unwrapDiscardedCall(statement.expression);
  if (!call) return false;
  const receiverType = getExpressionType(call.callee.object, typeCtx, false);
  const returnType = getExpressionType(call, typeCtx, false);
  return Boolean(
    receiverType &&
    returnType &&
    isPlaywrightType(receiverType, PLAYWRIGHT_RECEIVERS, typeCtx.ts) &&
    isPlaywrightType(returnType, PLAYWRIGHT_LOCATORS, typeCtx.ts),
  );
}

module.exports = {
  excludesUndefined,
  getExpressionType,
  getTypeContext,
  isAlwaysTruthy,
  isDiscardedPlaywrightLocator,
  isUncheckedIndexedAccess,
};
