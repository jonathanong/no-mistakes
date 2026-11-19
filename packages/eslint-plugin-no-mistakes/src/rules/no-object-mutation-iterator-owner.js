"use strict";

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
  const ts = require("typescript");
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
  return !declarations.every(
    (declaration) =>
      projectDeclaration(program, declaration) && returnsFreshCollection(declaration),
  );
}

function projectDeclaration(program, declaration) {
  const file = declaration.getSourceFile();
  return (
    !program.isSourceFileDefaultLibrary(file) && !program.isSourceFileFromExternalLibrary(file)
  );
}

const FRESH_CALLS = new Set(["map", "filter", "slice", "concat", "flat", "flatMap", "toSorted"]);
const FRESH_CONSTRUCTORS = new Set(["Map", "Set", "Array"]);

function returnsFreshCollection(declaration) {
  const ts = require("typescript");
  if (!ts.isMethodDeclaration(declaration) || !declaration.body) return false;
  if (declaration.asteriskToken) return false;
  const returns = [];
  const visit = (node) => {
    if (ts.isFunctionLike(node) || ts.isClassLike(node)) return;
    if (ts.isReturnStatement(node)) returns.push(node.expression);
    ts.forEachChild(node, visit);
  };
  ts.forEachChild(declaration.body, visit);
  return returns.length > 0 && returns.every((expression) => freshExpression(ts, expression));
}

function freshExpression(ts, expression) {
  if (!expression) return false;
  const node = ts.skipOuterExpressions(expression);
  if (ts.isArrayLiteralExpression(node)) return true;
  if (ts.isNewExpression(node)) {
    return ts.isIdentifier(node.expression) && FRESH_CONSTRUCTORS.has(node.expression.text);
  }
  if (!ts.isCallExpression(node) || !ts.isPropertyAccessExpression(node.expression)) return false;
  const { expression: owner, name } = node.expression;
  if (ts.isIdentifier(owner) && owner.text === "Array")
    return name.text === "from" || name.text === "of";
  return FRESH_CALLS.has(name.text);
}

module.exports = { yieldsCallerOwnedElements };
