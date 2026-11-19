"use strict";

// No program, or an unresolved type, keeps today's call form.
// typescript is an optional peer, so this module must load without it.
const CALLABLE = { callable: true, query: false };

function parserServices(context) {
  const services = context?.sourceCode?.parserServices;
  if (
    !services?.program ||
    services.program.getCompilerOptions().noResolve ||
    !services.esTreeNodeToTSNodeMap
  ) {
    return null;
  }
  return services;
}

function hasCallSignatures(checker, type, ts) {
  return checker.getSignaturesOfType(type, ts.SignatureKind.Call).length > 0;
}

function queryMethod(checker, type, ts) {
  const property = type.getProperty("query");
  const location = property?.valueDeclaration ?? property?.declarations?.[0];
  if (!location) return false;
  const queryType = checker.getNonNullableType(
    checker.getTypeOfSymbolAtLocation(property, location),
  );
  return hasCallSignatures(checker, queryType, ts);
}

function executorCallShape(node, context) {
  const services = parserServices(context);
  if (!services || !node) return CALLABLE;
  const ts = require("typescript");
  const tsNode = services.esTreeNodeToTSNodeMap.get(node);
  if (!tsNode) return CALLABLE;
  const checker = services.program.getTypeChecker();
  const type = checker.getNonNullableType(
    checker.getApparentType(checker.getTypeAtLocation(tsNode)),
  );
  if (type.flags & (ts.TypeFlags.Any | ts.TypeFlags.Unknown)) return CALLABLE;
  if (hasCallSignatures(checker, type, ts)) return CALLABLE;
  return { callable: false, query: queryMethod(checker, type, ts) };
}

module.exports = { executorCallShape };
