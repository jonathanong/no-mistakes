"use strict";

function createMutationCollector(checker, program, result, addSymbol) {
  const mutationApiAliases = new Map();
  const deferredMutationCalls = [];
  const mutationApiDeclarations = new Set();
  let usesMutationApi = false;

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
  function mutationApi(node) {
    const api =
      (builtinOwner(node, "Object", "assign") && "Object.assign") ||
      (builtinOwner(node, "Object", "defineProperty") && "Object.defineProperty") ||
      (builtinOwner(node, "Object", "defineProperties") && "Object.defineProperties") ||
      (builtinOwner(node, "Object", "setPrototypeOf") && "Object.setPrototypeOf") ||
      (builtinOwner(node, "Reflect", "set") && "Reflect.set");
    return api || null;
  }
  function visit(node) {
    if (node.initializer && node.name) {
      const api = mutationApi(node.initializer);
      const symbol = checker.getSymbolAtLocation(node.name);
      if (api && symbol) {
        rememberMutationApi(node.initializer);
        mutationApiAliases.set(symbol, api);
      }
    }
    if (!node.arguments?.length) return;
    const api = mutationApi(node.expression);
    if (api) {
      rememberMutationApi(node.expression);
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
  function finish(onUnknownCall) {
    for (const node of deferredMutationCalls) {
      const alias = checker.getSymbolAtLocation(node.expression);
      const api = mutationApiAliases.get(alias);
      if (api && !result.symbols.has(alias)) {
        applyApiMutation(node, api);
        continue;
      }
      onUnknownCall(node);
    }
    if (
      usesMutationApi &&
      [...mutationApiDeclarations].some((declaration) => result.declarations.has(declaration))
    ) {
      result.uncertainMutation = true;
    }
  }
  return { visit, finish };
}

module.exports = { createMutationCollector };
