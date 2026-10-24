"use strict";

// Borrow parser-owned nodes and symbols; each rule invocation owns this fact map.
function sourceWrites(source, checker, facts) {
  if (facts.has(source)) return facts.get(source);
  const result = { symbols: new Set(), eval: false };
  function written(node) {
    const symbol =
      checker.getShorthandAssignmentValueSymbol(node) ?? checker.getSymbolAtLocation(node);
    if (symbol) result.symbols.add(symbol);
    node.forEachChild(written);
  }
  function visit(node) {
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
    // Both for-in and for-of assign their initializer on every iteration.
    if (node.initializer && node.expression && node.getFirstToken().getText() === "for") {
      written(node.initializer);
    }
    let callee = node.arguments && node.expression;
    // Parentheses and TypeScript assertions preserve direct-eval semantics.
    while (callee?.expression && !callee.name && !callee.argumentExpression && !callee.arguments) {
      callee = callee.expression;
    }
    if (callee?.escapedText === "eval") result.eval = true;
    node.forEachChild(visit);
  }
  visit(source);
  facts.set(source, result);
  return result;
}

module.exports = { sourceWrites };
