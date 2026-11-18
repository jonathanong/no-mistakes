"use strict";

const { lookup } = require("./no-object-mutation-values");
const { methodName, unwrap } = require("./no-object-mutation-syntax");

function variableFor(sourceCode, node) {
  const reference = sourceCode.getScope(node).references.find((item) => item.identifier === node);
  return reference?.resolved || null;
}

function globalBuiltin(state, node, scope, name) {
  const target = unwrap(node);
  return (
    target?.type === "Identifier" &&
    target.name === name &&
    !lookup(scope, name) &&
    !variableFor(state.context.sourceCode, target)?.defs.length &&
    !state.builtins[name].iterationChanged
  );
}

function factoryCall(node, scope) {
  const call = unwrap(node);
  if (call?.type !== "CallExpression") return false;
  const callee = unwrap(call.callee);
  const object = unwrap(callee?.object);
  if (callee?.type !== "MemberExpression" || object?.type !== "Identifier") return false;
  const name = lookup(scope, object.name)?.tag.intrinsic || object.name;
  return (
    (name === "Object" && ["entries", "keys", "values"].includes(methodName(callee))) ||
    (name === "Array" && methodName(callee) === "from")
  );
}

function checkedArray(state, expression, scope, method, proven, seen = new Set()) {
  const sourceCode = state.context.sourceCode;
  const services = sourceCode.parserServices;
  if (!services?.program || !services.esTreeNodeToTSNodeMap) return null;
  const node = unwrap(expression);
  if (!proven && factoryCall(node, scope)) return false;
  if (node.type === "Identifier") {
    const variable = variableFor(sourceCode, node);
    if (seen.has(variable)) return false;
    if (!proven && variable?.references.some((reference) => reference.isWrite() && !reference.init))
      return false;
    const definition = variable?.defs?.[0]?.node;
    if (!proven && definition?.type === "VariableDeclarator" && definition.init) {
      seen.add(variable);
      if (!checkedArray(state, definition.init, scope, method, proven, seen)) return false;
    }
  }
  const tsNode = services.esTreeNodeToTSNodeMap.get(node);
  if (!tsNode) return false;
  const checker = services.program.getTypeChecker();
  function array(type) {
    if (type.isUnion()) return type.types.every(array);
    if (!checker.isArrayType(type) && !checker.isTupleType(type)) return false;
    const declarations = checker.getPropertyOfType(type, method)?.declarations;
    return Boolean(
      declarations?.length &&
      declarations.every((declaration) =>
        services.program.isSourceFileDefaultLibrary(declaration.getSourceFile()),
      ),
    );
  }
  return array(checker.getTypeAtLocation(tsNode));
}

module.exports = { checkedArray, globalBuiltin };
