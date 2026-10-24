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
    if (
      (node.name?.text ?? node.argumentExpression?.text) === "prototype" &&
      builtinCollection(checker.getTypeAtLocation(node))
    ) {
      result.uncertainMutation = true;
      return;
    }
    if (node.escapedText !== undefined) {
      const symbol =
        checker.getShorthandAssignmentValueSymbol(node.parent) ?? checker.getSymbolAtLocation(node);
      if (builtinCollection(checker.getTypeAtLocation(node))) {
        if (symbol && !seenSymbols.has(symbol)) {
          seenSymbols.add(symbol);
          result.escapedSymbols.add(symbol);
          const declaration = symbol.valueDeclaration;
          if (declaration?.initializer) escapeValue(declaration.initializer, seenSymbols);
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
    // Compiler expression wrappers include assertions, parentheses and spreads.
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
      for (const element of node.elements) escapeValue(element, seenSymbols);
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
  function collectionMethod(node) {
    const callee = node.expression;
    const wrapper = ["call", "apply"].includes(callee?.name?.text) ? callee.name.text : null;
    const method = wrapper ? callee.expression : callee;
    const collection =
      method?.name &&
      method.expression &&
      builtinCollection(checker.getTypeAtLocation(method.expression));
    if (!collection) return null;
    const name = method.name.text;
    const valid =
      collection === "Map" || collection === "WeakMap"
        ? ["get", "set", "has", "delete"].includes(name)
        : ["has", "delete", "add"].includes(name);
    return valid ? { collection, name, wrapper } : null;
  }
  function visit(node) {
    if (["=", "&&=", "||=", "??="].includes(node.operatorToken?.getText()) && node.right)
      escapeValue(node.right);
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
    const method = collectionMethod(node);
    if (method) {
      const valueIndexes = method.name === "set" ? [0, 1] : method.name === "add" ? [0] : [];
      if (!valueIndexes.length) return;
      if (method.wrapper === "call") {
        for (const index of valueIndexes) escapeValue(node.arguments[index + 1]);
      } else if (method.wrapper === "apply") {
        const args = node.arguments[1];
        const elements = args?.elements;
        if (Array.isArray(elements) && !elements.some((element) => element.dotDotDotToken)) {
          for (const index of valueIndexes) escapeValue(elements[index]);
        } else escapeValue(args);
      } else for (const index of valueIndexes) escapeValue(node.arguments[index]);
      return;
    }
    for (const argument of node.arguments) escapeValue(argument);
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
