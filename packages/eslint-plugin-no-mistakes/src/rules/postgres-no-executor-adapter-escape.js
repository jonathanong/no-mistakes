"use strict";

function createCollectionEscape(checker, program, result) {
  const escapedValueNodes = new WeakSet();
  const collectionAliases = [];

  function builtinCollection(type) {
    const symbol = type?.symbol;
    const name = symbol?.getName();
    return ["Map", "WeakMap", "Set", "WeakSet"].includes(name) &&
      symbol.declarations?.length &&
      symbol.declarations.every((declaration) =>
        program.isSourceFileDefaultLibrary(declaration.getSourceFile()),
      )
      ? name
      : null;
  }
  function escapeValue(node, seenSymbols = new Set()) {
    if (!node || escapedValueNodes.has(node)) return;
    escapedValueNodes.add(node);
    if (node.getFirstToken?.()?.getText() === "typeof") return;
    if (node.escapedText !== undefined) {
      const symbol =
        checker.getShorthandAssignmentValueSymbol(node.parent) ?? checker.getSymbolAtLocation(node);
      if (builtinCollection(checker.getTypeAtLocation(node))) {
        if (symbol && !seenSymbols.has(symbol)) {
          seenSymbols.add(symbol);
          result.escapedSymbols.add(symbol);
          const declaration = symbol.valueDeclaration;
          if (declaration?.initializer?.escapedText !== undefined)
            escapeValue(declaration.initializer, seenSymbols);
        }
      } else if (symbol && !seenSymbols.has(symbol)) {
        seenSymbols.add(symbol);
        const initializer = symbol.valueDeclaration?.initializer;
        if (initializer) escapeValue(initializer, seenSymbols);
      }
      return;
    }
    if (node.equalsGreaterThanToken && node.body && !node.body.statements) {
      escapeValue(node.body, seenSymbols);
      return;
    }
    if (
      node.expression &&
      !node.name &&
      !node.arguments &&
      !node.argumentExpression &&
      !node.operatorToken &&
      !node.typeArguments
    ) {
      escapeValue(node.expression, seenSymbols);
    } else if (Array.isArray(node.elements)) {
      for (const element of node.elements)
        escapeValue(element.dotDotDotToken ? element.expression : element, seenSymbols);
    } else if (Array.isArray(node.properties)) {
      for (const property of node.properties) {
        if (property.initializer) escapeValue(property.initializer, seenSymbols);
        else if (property.name?.escapedText !== undefined) escapeValue(property.name, seenSymbols);
        else if (property.expression) escapeValue(property.expression, seenSymbols);
      }
    } else if (node.whenTrue && node.whenFalse) {
      escapeValue(node.whenTrue, seenSymbols);
      escapeValue(node.whenFalse, seenSymbols);
    }
  }
  function knownCollectionMethod(node) {
    const callee = node.expression;
    if (
      callee?.name &&
      callee.expression &&
      builtinCollection(checker.getTypeAtLocation(callee.expression))
    )
      return ["get", "set", "has", "delete", "add"].includes(callee.name.text);
    const method = callee?.expression;
    return Boolean(
      ["call", "apply"].includes(callee?.name?.text) &&
      method?.name &&
      method.expression &&
      builtinCollection(checker.getTypeAtLocation(method.expression)) &&
      ["get", "set", "has", "delete", "add"].includes(method.name.text),
    );
  }
  function visit(node) {
    if (node.initializer && node.name) {
      const initializer = node.initializer;
      const sourceType = builtinCollection(checker.getTypeAtLocation(initializer));
      const boundType = builtinCollection(checker.getTypeAtLocation(node.name));
      if (
        (sourceType && sourceType !== boundType) ||
        Array.isArray(initializer.properties) ||
        Array.isArray(initializer.elements) ||
        initializer.equalsGreaterThanToken
      )
        escapeValue(initializer);
    }
    if (node.expression && node.getFirstToken?.()?.getText() === "return")
      escapeValue(node.expression);
    if (node.equalsGreaterThanToken && node.body && !node.body.statements) escapeValue(node.body);
    if (node.expression && node.getFirstToken?.()?.getText() === "export")
      escapeValue(node.expression);
    if (node.parent && node.parent.parent?.exportClause === node.parent) {
      const exported = checker.getExportSpecifierLocalTargetSymbol?.(node);
      if (exported?.valueDeclaration) result.escapedSymbols.add(exported);
    }
    if (node.declarationList && node.modifiers?.some((modifier) => modifier.getText() === "export"))
      for (const declaration of node.declarationList.declarations) {
        const symbol = checker.getSymbolAtLocation(declaration.name);
        if (symbol) result.escapedSymbols.add(symbol);
      }
    if (node.initializer?.escapedText === undefined || node.name?.escapedText === undefined) return;
    const source = checker.getSymbolAtLocation(node.initializer),
      alias = checker.getSymbolAtLocation(node.name);
    const type = builtinCollection(checker.getTypeAtLocation(node.initializer));
    if (source && alias && type && builtinCollection(checker.getTypeAtLocation(node.name)) === type)
      collectionAliases.push([source, alias]);
  }
  function unknownCall(node) {
    if (knownCollectionMethod(node)) return;
    for (const argument of node.arguments)
      escapeValue(argument.dotDotDotToken ? argument.expression : argument);
  }
  function finish() {
    const aliases = new Map();
    for (const [source, alias] of collectionAliases) {
      if (!aliases.has(source)) aliases.set(source, new Set());
      if (!aliases.has(alias)) aliases.set(alias, new Set());
      aliases.get(source).add(alias);
      aliases.get(alias).add(source);
    }
    const escaped = [...result.escapedSymbols];
    for (let index = 0; index < escaped.length; index++) {
      for (const alias of aliases.get(escaped[index]) ?? [])
        if (!result.escapedSymbols.has(alias)) {
          result.escapedSymbols.add(alias);
          escaped.push(alias);
        }
    }
  }
  return { visit, unknownCall, finish };
}

module.exports = { createCollectionEscape };
