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
  function written(node) {
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
