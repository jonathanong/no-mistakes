"use strict";

function referenceFacts(fn, checker, facts) {
  let cached = facts.get(fn);
  if (cached) return cached;
  const refs = new Map();
  let opaqueScope = false;
  function add(symbol, node) {
    if (!symbol) return;
    if (!refs.has(symbol)) refs.set(symbol, []);
    refs.get(symbol).push(node);
  }
  function visit(node) {
    if (node.arguments && node.expression?.escapedText === "eval") opaqueScope = true;
    if (node.escapedText === "arguments") opaqueScope = true;
    add(checker.getShorthandAssignmentValueSymbol(node), node);
    if (typeof node.escapedText === "string") {
      add(checker.getSymbolAtLocation(node), node);
    }
    node.forEachChild(visit);
  }
  for (const parameter of fn.parameters) visit(parameter);
  visit(fn.body);
  cached = { refs, opaqueScope, verdicts: new Map() };
  facts.set(fn, cached);
  return cached;
}

function bindingName(param, property) {
  if (typeof param?.name?.escapedText === "string") {
    return { node: param.name, object: true };
  }
  const element = param?.name?.elements?.find(
    (item) =>
      !item.initializer &&
      !item.dotDotDotToken &&
      typeof item.name?.escapedText === "string" &&
      (item.propertyName?.text ?? item.name.text) === property,
  );
  return { node: element.name, object: false };
}

function staticKey(member) {
  if (member.name) return member.name.text;
  const key = member.argumentExpression;
  const raw = key?.getText();
  if (
    typeof key?.text === "string" &&
    raw.length >= 2 &&
    (raw[0] === '"' || raw[0] === "'" || raw[0] === "`") &&
    raw.at(-1) === raw[0]
  )
    return key.text;
  return null;
}

function constBinding(declaration, checker) {
  if (
    !declaration?.parent?.declarations ||
    declaration.parent.getFirstToken()?.getText() !== "const" ||
    typeof declaration.name?.escapedText !== "string"
  )
    return null;
  return checker.getSymbolAtLocation(declaration.name);
}

// SQL discovery unwraps type and parenthesis wrappers, but not await or yield.
function followedExpression(node) {
  let expression = node;
  let parent = expression.parent;
  while (
    parent?.expression === expression &&
    !parent.arguments &&
    !parent.name &&
    !parent.argumentExpression &&
    !parent.initializer &&
    (parent.type ||
      parent.typeArguments ||
      parent.getFirstToken()?.getText() === "(" ||
      parent.getLastToken()?.getText() === "!")
  ) {
    expression = parent;
    parent = expression.parent;
  }
  return { expression, parent };
}

function directCall(node) {
  const { expression, parent } = followedExpression(node);
  return Boolean(
    parent?.expression === expression &&
    Array.isArray(parent.arguments) &&
    parent.getFirstToken()?.getText() !== "new" &&
    parent.arguments.every((argument) => argument.getFirstToken()?.getText() !== "..."),
  );
}

function judgedMethod(node) {
  const method = node.parent;
  if (method?.expression !== node || method.questionDotToken) return false;
  const name = staticKey(method);
  if (name !== "call" && name !== "apply") return false;
  const call = method.parent;
  if (
    call?.expression !== method ||
    !Array.isArray(call.arguments) ||
    call.getFirstToken()?.getText() === "new" ||
    call.arguments.some((argument) => argument.getFirstToken()?.getText() === "...")
  )
    return false;
  if (name === "call") return true;
  const array = call.arguments[1];
  return Boolean(
    Array.isArray(array?.elements) &&
    array.elements.every(
      (element) => element.getText() !== "" && element.getFirstToken()?.getText() !== "...",
    ),
  );
}

function valueUse(node, property, checker, state) {
  if (node.parent?.expression === node && node.parent.getFirstToken()?.getText() === "typeof")
    return true;
  if (directCall(node)) return true;
  if (judgedMethod(node)) return true;
  const { expression, parent } = followedExpression(node);
  if (parent?.initializer !== expression) return false;
  const alias = constBinding(parent, checker);
  return Boolean(alias && safeSymbol(alias, false, property, checker, state));
}

function safeSymbol(symbol, object, property, checker, state) {
  for (const identifier of state.refs.get(symbol) ?? []) {
    const parent = identifier.parent;
    if (
      parent?.exprName === identifier ||
      (parent?.name === identifier &&
        (parent.parent?.parameters || parent.parent?.declarations || parent.parent?.elements))
    )
      continue;
    if (parent?.expression === identifier && parent.getFirstToken()?.getText() === "typeof")
      continue;
    if (object) {
      if (parent?.expression === identifier && (parent.name || parent.argumentExpression)) {
        const name = staticKey(parent);
        if (name == null) return false;
        if (name !== property) continue;
        if (valueUse(parent, property, checker, state)) continue;
      }
    } else if (valueUse(identifier, property, checker, state)) {
      continue;
    }
    return false;
  }
  return true;
}

function tsOptionBody(fn, index, property, checker, facts) {
  if (!fn?.body) return false;
  const state = referenceFacts(fn, checker, facts);
  const key = `${index}:${property}`;
  if (state.verdicts.has(key)) return state.verdicts.get(key);
  const binding = bindingName(fn.parameters[index], property);
  const symbol = checker.getSymbolAtLocation(binding.node);
  const safe = Boolean(
    symbol && !state.opaqueScope && safeSymbol(symbol, binding.object, property, checker, state),
  );
  state.verdicts.set(key, safe);
  return safe;
}

module.exports = { tsOptionBody };
