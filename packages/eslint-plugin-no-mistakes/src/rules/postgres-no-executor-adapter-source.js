"use strict";
function sourceWrites(source, checker, facts, program) {
  if (facts.has(source)) return facts.get(source);
  const result = {
    symbols: new Set(),
    declarations: new Set(),
    escapedSymbols: new Set(),
    uncertainMutation: false,
    eval: false,
  };
  const mutationApiAliases = new Map();
  const deferredMutationCalls = [];
  const mutationApiDeclarations = new Set();
  const escapedValueNodes = new WeakSet();
  const collectionAliases = [];
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
    for (const declaration of checker.getSymbolAtLocation(callee.name)?.declarations ?? [])
      mutationApiDeclarations.add(declaration);
  }
  function mutateTarget(target, property) {
    if (!target) return;
    const type = checker.getTypeAtLocation(target);
    const uncertainType = () =>
      type.isUnion?.() || ["any", "unknown"].includes(checker.typeToString(type));
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
    if (node.escapedText !== undefined) {
      if (builtinCollection(checker.getTypeAtLocation(node))) {
        const symbol =
          checker.getShorthandAssignmentValueSymbol(node.parent) ??
          checker.getSymbolAtLocation(node);
        if (symbol && !seenSymbols.has(symbol)) {
          seenSymbols.add(symbol);
          result.escapedSymbols.add(symbol);
          const declaration = symbol.valueDeclaration;
          if (declaration?.initializer?.escapedText !== undefined)
            escapeValue(declaration.initializer, seenSymbols);
        }
      }
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
  function apiMutation(node) {
    if (!node.arguments?.length) return;
    const callee = node.expression;
    const api =
      (builtinOwner(callee, "Object", "assign") && "Object.assign") ||
      (builtinOwner(callee, "Object", "defineProperty") && "Object.defineProperty") ||
      (builtinOwner(callee, "Object", "defineProperties") && "Object.defineProperties") ||
      (builtinOwner(callee, "Object", "setPrototypeOf") && "Object.setPrototypeOf") ||
      (builtinOwner(callee, "Reflect", "set") && "Reflect.set");
    if (api) {
      rememberMutationApi(callee);
      applyApiMutation(node, api);
      return;
    }
    deferredMutationCalls.push(node);
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
          const name = staticPropertyName(property.name, true);
          if (property.expression || name == null) {
            mutateTarget(target, null);
            break;
          }
          mutateTarget(target, name);
        }
      }
      return;
    }
    if (api === "Object.defineProperties") {
      const target = node.arguments[0],
        descriptors = node.arguments[1]?.properties;
      if (!descriptors) {
        mutateTarget(target, null);
        return;
      }
      for (const descriptor of descriptors) {
        const name = staticPropertyName(descriptor.name, true);
        if (descriptor.expression || name == null) {
          mutateTarget(target, null);
          return;
        }
        mutateTarget(target, name);
      }
      return;
    }
    if (api === "Object.setPrototypeOf") {
      mutateTarget(node.arguments[0], null);
      return;
    }
    if ((api === "Object.defineProperty" || api === "Reflect.set") && node.arguments.length > 1)
      mutateTarget(node.arguments[0], staticPropertyName(node.arguments[1]));
  }
  function visit(node) {
    if (node.initializer?.escapedText !== undefined && node.name?.escapedText !== undefined) {
      const source = checker.getSymbolAtLocation(node.initializer),
        alias = checker.getSymbolAtLocation(node.name);
      const type = builtinCollection(checker.getTypeAtLocation(node.initializer));
      if (
        source &&
        alias &&
        type &&
        builtinCollection(checker.getTypeAtLocation(node.name)) === type
      )
        collectionAliases.push([source, alias]);
    }
    if (node.initializer && node.name) {
      const initializer = node.initializer;
      const api =
        (builtinOwner(initializer, "Object", "assign") && "Object.assign") ||
        (builtinOwner(initializer, "Object", "defineProperty") && "Object.defineProperty") ||
        (builtinOwner(initializer, "Object", "defineProperties") && "Object.defineProperties") ||
        (builtinOwner(initializer, "Object", "setPrototypeOf") && "Object.setPrototypeOf") ||
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
  for (const node of deferredMutationCalls) {
    const alias = checker.getSymbolAtLocation(node.expression);
    const api = mutationApiAliases.get(alias);
    if (api && !result.symbols.has(alias)) {
      applyApiMutation(node, api);
      continue;
    }
    if (!knownCollectionMethod(node)) {
      for (const argument of node.arguments) {
        escapeValue(argument.dotDotDotToken ? argument.expression : argument);
      }
    }
  }
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
