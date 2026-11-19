"use strict";

const ts = require("typescript");

function yieldsCallerOwnedElements(state, expression, method) {
  const services = state.context.sourceCode.parserServices;
  if (!services?.program || !services.esTreeNodeToTSNodeMap) return true;
  const tsNode = services.esTreeNodeToTSNodeMap.get(expression);
  if (!tsNode) return true;
  const checker = services.program.getTypeChecker();
  return callerOwnedType(
    checker,
    services.program,
    checker.getTypeAtLocation(tsNode),
    method,
    new Set(),
  );
}

function callerOwnedType(checker, program, type, method, seen) {
  if (!type || seen.has(type)) return true;
  seen.add(type);
  if (type.flags & (ts.TypeFlags.Any | ts.TypeFlags.Unknown)) return true;
  if (type.isUnion()) {
    return type.types.some((part) => callerOwnedType(checker, program, part, method, seen));
  }
  if (type.flags & ts.TypeFlags.TypeParameter) {
    const constraint = type.getConstraint?.() ?? checker.getBaseConstraintOfType(type);
    return constraint ? callerOwnedType(checker, program, constraint, method, seen) : true;
  }
  const apparent = type.isIntersection() ? type : checker.getApparentType(type);
  return callerOwnedMembers(checker, program, apparent, method);
}

function callerOwnedMembers(checker, program, type, method) {
  const declarations = checker.getPropertyOfType(type, method)?.declarations;
  if (!declarations?.length) return true;
  return !declarations.every((declaration) => projectDeclaration(program, declaration));
}

function projectDeclaration(program, declaration) {
  const file = declaration.getSourceFile();
  return (
    !program.isSourceFileDefaultLibrary(file) && !program.isSourceFileFromExternalLibrary(file)
  );
}

module.exports = { yieldsCallerOwnedElements };
