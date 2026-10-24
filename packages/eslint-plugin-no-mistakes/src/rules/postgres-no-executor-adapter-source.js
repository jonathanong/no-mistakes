"use strict";

// Borrow parser-owned nodes and symbols; each rule invocation owns this fact map.
function sourceWrites(source, checker, facts, program) {
  if (facts.has(source)) return facts.get(source);
  const result = {
    symbols: new Set(),
    declarations: new Set(),
    uncertainMutation: false,
    eval: false,
  };
  const mutationApiAliases = new Map();
  const deferredMutationCalls = [];
  const mutationApiDeclarations = new Set();
  let usesMutationApi = false;
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
  function builtinOwner(node, name, member) {
    const owner = node.expression;
    if (node.name?.text !== member || owner?.escapedText !== name) return false;
    const symbol = checker.getSymbolAtLocation(owner);
    return Boolean(
      symbol?.declarations?.length &&
      symbol.declarations.every(
        (declaration) =>
          program.isSourceFileDefaultLibrary(declaration.getSourceFile()) &&
          declaration.name?.text === name,
      ),
    );
  }
  function literalPropertyName(node) {
    const text = node?.getText?.();
    return typeof node?.text === "string" &&
      typeof text === "string" &&
      ((text.startsWith('"') && text.endsWith('"')) || (text.startsWith("'") && text.endsWith("'")))
      ? node.text
      : null;
  }
  function staticPropertyName(name, allowIdentifier = false) {
    if (allowIdentifier && typeof name?.escapedText === "string") return name.escapedText;
    return literalPropertyName(name) ?? literalPropertyName(name?.expression);
  }
  function rememberMutationApi(callee) {
    usesMutationApi = true;
    const symbol = checker.getSymbolAtLocation(callee.name);
    for (const declaration of symbol?.declarations ?? []) {
      mutationApiDeclarations.add(declaration);
    }
  }
  function mutateTarget(target, property) {
    if (!target) return;
    const type = checker.getTypeAtLocation(target);
    function uncertainType() {
      return type.isUnion?.() || ["any", "unknown"].includes(checker.typeToString(type));
    }
    if (property == null) {
      const properties = checker.getPropertiesOfType(type);
      if (!properties.length && uncertainType()) result.uncertainMutation = true;
      for (const symbol of properties) addSymbol(symbol);
      return;
    }
    const symbol = checker.getPropertyOfType(type, property);
    if (symbol) addSymbol(symbol);
    else if (uncertainType()) result.uncertainMutation = true;
  }
  function apiMutation(node) {
    if (!node.arguments?.length) return;
    const callee = node.expression;
    const api =
      (builtinOwner(callee, "Object", "assign") && "Object.assign") ||
      (builtinOwner(callee, "Object", "defineProperty") && "Object.defineProperty") ||
      (builtinOwner(callee, "Reflect", "set") && "Reflect.set");
    if (api) {
      rememberMutationApi(callee);
      applyApiMutation(node, api);
      return;
    }
    if (callee?.escapedText) deferredMutationCalls.push(node);
  }
  function applyApiMutation(node, api) {
    if (api === "Object.assign") {
      const target = node.arguments[0];
      for (const sourceArgument of node.arguments.slice(1)) {
        const properties = sourceArgument.properties;
        if (!properties) {
          mutateTarget(target, null);
          continue;
        }
        for (const property of properties) {
          if (property.expression) {
            mutateTarget(target, null);
            break;
          }
          const name = staticPropertyName(property.name, true);
          if (name == null) mutateTarget(target, null);
          else mutateTarget(target, name);
        }
      }
      return;
    }
    if ((api === "Object.defineProperty" || api === "Reflect.set") && node.arguments.length > 1) {
      mutateTarget(node.arguments[0], staticPropertyName(node.arguments[1]));
    }
  }
  function visit(node) {
    if (node.initializer && node.name) {
      const initializer = node.initializer;
      const api =
        (builtinOwner(initializer, "Object", "assign") && "Object.assign") ||
        (builtinOwner(initializer, "Object", "defineProperty") && "Object.defineProperty") ||
        (builtinOwner(initializer, "Reflect", "set") && "Reflect.set");
      const symbol = checker.getSymbolAtLocation(node.name);
      if (api && symbol) {
        rememberMutationApi(initializer);
        mutationApiAliases.set(symbol, api);
      }
    }
    apiMutation(node);
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
  for (const node of deferredMutationCalls) {
    const alias = checker.getSymbolAtLocation(node.expression);
    const api = mutationApiAliases.get(alias);
    if (api && !result.symbols.has(alias)) {
      applyApiMutation(node, api);
      continue;
    }
    for (const argument of node.arguments) {
      mutateTarget(argument.dotDotDotToken ? argument.expression : argument, null);
    }
  }
  if (
    usesMutationApi &&
    [...mutationApiDeclarations].some((declaration) => result.declarations.has(declaration))
  ) {
    result.uncertainMutation = true;
  }
  facts.set(source, result);
  return result;
}

module.exports = { sourceWrites };
