"use strict";

const { createMutationCollector } = require("./postgres-no-executor-adapter-mutation");
const { createCollectionEscape } = require("./postgres-no-executor-adapter-escape");

// Borrow parser-owned nodes and symbols. One source visit builds one cached fact map.
function sourceWrites(source, checker, facts, program) {
  if (facts.has(source)) return facts.get(source);
  const result = {
    symbols: new Set(),
    declarations: new Set(),
    escapedSymbols: new Set(),
    uncertainMutation: false,
    eval: false,
  };
  function addSymbol(symbol) {
    if (!symbol) return;
    result.symbols.add(symbol);
    for (const declaration of symbol.declarations ?? []) result.declarations.add(declaration);
  }
  function unwrap(node) {
    while (
      node?.expression &&
      !node.name &&
      !node.argumentExpression &&
      !node.arguments &&
      !node.operatorToken
    )
      node = node.expression;
    return node;
  }
  function writtenCollectionMember(node) {
    const target = unwrap(node);
    if (!target?.expression) return;
    const argument = target.argumentExpression;
    const text = argument?.getText?.();
    const computedName =
      typeof argument?.text === "string" &&
      ((text?.startsWith('"') && text.endsWith('"')) ||
        (text?.startsWith("'") && text.endsWith("'")))
        ? argument.text
        : null;
    const property = target.name?.escapedText ?? computedName;
    const receiver = unwrap(target.expression);
    if (receiver?.escapedText === "globalThis") {
      const symbol = checker.getSymbolAtLocation(receiver);
      const globalType = checker.getTypeAtLocation(receiver);
      const constructor = property && checker.getPropertyOfType(globalType, property);
      const unknownProperty = property == null;
      const builtinConstructor =
        ["Map", "WeakMap", "Set", "WeakSet"].includes(property) &&
        constructor?.declarations?.length &&
        constructor.declarations.every(
          (declaration) =>
            program.isSourceFileDefaultLibrary(declaration.getSourceFile()) &&
            declaration.name?.text === property,
        );
      if (
        symbol?.getName() === "globalThis" &&
        !symbol.declarations?.length &&
        (unknownProperty || builtinConstructor)
      )
        result.uncertainMutation = true;
      return;
    }
    const type = checker.getTypeAtLocation(receiver),
      collection = type?.symbol;
    const name = collection?.getName();
    if (
      !["Map", "WeakMap", "Set", "WeakSet"].includes(name) ||
      !collection.declarations?.length ||
      !collection.declarations.every(
        (declaration) =>
          program.isSourceFileDefaultLibrary(declaration.getSourceFile()) &&
          declaration.name?.text === name,
      )
    )
      return;
    const methods =
      property === "__proto__" || property == null
        ? checker.getPropertiesOfType(type)
        : [checker.getPropertyOfType(type, property)].filter(Boolean);
    for (const method of methods) addSymbol(method);
  }
  function written(node) {
    writtenCollectionMember(node);
    const symbol =
      checker.getShorthandAssignmentValueSymbol(node) ?? checker.getSymbolAtLocation(node);
    addSymbol(symbol);
    node.forEachChild(written);
  }
  const mutation = createMutationCollector(checker, program, result, addSymbol);
  const escape = createCollectionEscape(checker, program, result);
  function visit(node) {
    escape.visit(node);
    mutation.visit(node);
    const operator = node.operatorToken?.getText();
    if (
      operator &&
      /^(?:=|\+=|-=|\*=|\*\*=|\/=|%=|<<=|>>=|>>>=|&=|\|=|\^=|&&=|\|\|=|\?\?=)$/.test(operator)
    ) {
      written(node.left);
    }
    if (node.operand && ["++", "--"].includes(node.getFirstToken().getText()))
      written(node.operand);
    if (node.operand && ["++", "--"].includes(node.getLastToken().getText())) written(node.operand);
    if (node.expression && node.getFirstToken().getText() === "delete") written(node.expression);
    if (node.initializer && node.expression && node.getFirstToken().getText() === "for")
      written(node.initializer);
    let callee = node.arguments && node.expression;
    while (callee?.expression && !callee.name && !callee.argumentExpression && !callee.arguments)
      callee = callee.expression;
    if (callee?.escapedText === "eval") result.eval = true;
    node.forEachChild(visit);
  }
  visit(source);
  mutation.finish(escape.unknownCall);
  escape.finish();
  facts.set(source, result);
  return result;
}

module.exports = { sourceWrites };
