"use strict";

const { resolveVariable } = require("./postgres-query-text");
const { sourceWrites } = require("./postgres-no-executor-adapter-source");
const ts = require("typescript");

const COLLECTION_METHODS = new Map([
  ["Map", new Set(["get", "set", "has", "delete"])],
  ["WeakMap", new Set(["get", "set", "has", "delete"])],
  ["Set", new Set(["has", "delete"])],
  ["WeakSet", new Set(["has", "delete"])],
]);

function stableReceiverBinding(receiver, context) {
  if (receiver?.type !== "Identifier") return false;
  const variable = resolveVariable(receiver, context);
  if (!variable || variable.defs.length !== 1) return false;
  if (variable.references.some((reference) => reference.isWrite() && !reference.init)) return false;
  const def = variable.defs[0];
  return def.type === "Variable" || def.type === "Parameter" ? def : false;
}

function builtinTypeDeclaration(declaration, name, program) {
  return (
    program.isSourceFileDefaultLibrary(declaration.getSourceFile()) &&
    declaration.name?.text === name
  );
}

function builtinCollection(type, program) {
  if (!type || type.isUnion?.() || type.isIntersection?.()) return null;
  const symbol = type.symbol;
  if (!symbol || !COLLECTION_METHODS.has(symbol.getName())) return null;
  const name = symbol.getName();
  if (
    !symbol.declarations?.length ||
    !symbol.declarations.every((entry) => builtinTypeDeclaration(entry, name, program))
  )
    return null;
  return name;
}

function stableCollectionValue(receiver, collection, context, checker, services, seen) {
  const def = stableReceiverBinding(receiver, context);
  if (!def) return false;
  if (def.type === "Parameter") return false;
  if (seen.has(def.node) || !def.node.init) return false;
  seen.add(def.node);
  if (def.node.init.type === "Identifier") {
    return stableCollectionValue(def.node.init, collection, context, checker, services, seen);
  }
  if (
    def.node.init.type !== "NewExpression" ||
    def.node.init.callee.type !== "Identifier" ||
    def.node.init.callee.name !== collection
  )
    return false;
  const tsInit = services.esTreeNodeToTSNodeMap.get(def.node.init);
  const tsCallee = services.esTreeNodeToTSNodeMap.get(def.node.init.callee);
  const constructor = checker.getSymbolAtLocation(tsCallee);
  return Boolean(
    tsInit &&
    constructor?.getName() === collection &&
    constructor.declarations?.length &&
    constructor.declarations.every((declaration) =>
      services.program.isSourceFileDefaultLibrary(declaration.getSourceFile()),
    ) &&
    builtinCollection(checker.getTypeAtLocation(tsInit), services.program) === collection,
  );
}

function allowsCollectionKey(call, index, context, facts) {
  if (
    call?.type !== "CallExpression" ||
    index !== 0 ||
    call.optional ||
    call.arguments.some((argument) => argument.type === "SpreadElement")
  )
    return false;
  const callee = call.callee;
  if (
    callee?.type !== "MemberExpression" ||
    callee.computed ||
    callee.optional ||
    callee.property.type !== "Identifier" ||
    !stableReceiverBinding(callee.object, context)
  )
    return false;
  const method = callee.property.name;
  const services = context.sourceCode.parserServices;
  if (!services.program || !services.esTreeNodeToTSNodeMap) return false;
  const checker = services.program.getTypeChecker();
  const tsReceiver = services.esTreeNodeToTSNodeMap.get(callee.object);
  const tsMethod = services.esTreeNodeToTSNodeMap.get(callee.property);
  if (!tsReceiver || !tsMethod) return false;
  const receiverType = checker.getTypeAtLocation(tsReceiver);
  const collection = builtinCollection(receiverType, services.program);
  if (!collection || !COLLECTION_METHODS.get(collection).has(method)) return false;
  if (!stableCollectionValue(callee.object, collection, context, checker, services, new Set()))
    return false;
  const methodSymbol = checker.getSymbolAtLocation(tsMethod);
  const writes = sourceWrites(tsReceiver.getSourceFile(), checker, facts, services.program);
  const receiverSymbol = checker.getSymbolAtLocation(tsReceiver);
  const constructor = checker
    .getSymbolsInScope(tsReceiver, ts.SymbolFlags.Value)
    .find(
      (symbol) =>
        symbol.getName() === collection &&
        symbol.declarations?.length &&
        symbol.declarations.every((declaration) =>
          builtinTypeDeclaration(declaration, collection, services.program),
        ),
    );
  const constructorMutated =
    constructor &&
    (writes.symbols.has(constructor) ||
      constructor.declarations.some((declaration) => writes.declarations.has(declaration)));
  return (
    !writes.uncertainMutation &&
    !writes.eval &&
    !constructorMutated &&
    !writes.escapedSymbols.has(receiverSymbol) &&
    methodSymbol?.declarations?.length &&
    !writes.symbols.has(methodSymbol) &&
    !methodSymbol.declarations.some((declaration) => writes.declarations.has(declaration))
  );
}

module.exports = { allowsCollectionKey };
