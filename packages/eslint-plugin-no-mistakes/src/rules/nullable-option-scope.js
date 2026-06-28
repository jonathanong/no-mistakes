"use strict";

function createScope(kind) {
  return {
    bindings: new Set(),
    kind,
    nullableBindings: new Set(),
    objectProps: new Map(),
  };
}

function variableScope(scopes, currentScope, node) {
  if (!node.parent || node.parent.kind !== "var") return currentScope();
  for (let index = scopes.length - 1; index >= 0; index -= 1) {
    if (scopes[index].kind === "function" || scopes[index].kind === "program") {
      return scopes[index];
    }
  }
  return currentScope();
}

function objectProps(scopes, name) {
  for (let index = scopes.length - 1; index >= 0; index -= 1) {
    if (scopes[index].bindings.has(name) && !scopes[index].objectProps.has(name)) return null;
    const props = scopes[index].objectProps.get(name);
    if (props) return props;
  }
  return null;
}

function isNullableBinding(scopes, name) {
  for (let index = scopes.length - 1; index >= 0; index -= 1) {
    if (scopes[index].bindings.has(name)) {
      return scopes[index].nullableBindings.has(name);
    }
  }
  return false;
}

function clearNullableBinding(scopes, name) {
  const scope = bindingScope(scopes, name);
  if (scope) scope.nullableBindings.delete(name);
}

function bindingScope(scopes, name) {
  for (let index = scopes.length - 1; index >= 0; index -= 1) {
    if (scopes[index].bindings.has(name)) return scopes[index];
  }
  return null;
}

function functionScopeVisitors(enter, exit) {
  return Object.fromEntries(
    ["FunctionDeclaration", "FunctionExpression", "ArrowFunctionExpression"].flatMap((key) => [
      [key, enter],
      [`${key}:exit`, exit],
    ]),
  );
}

function lexicalScopeVisitors(enter, exit) {
  return Object.fromEntries(
    [
      "BlockStatement",
      "ForStatement",
      "ForInStatement",
      "ForOfStatement",
      "SwitchStatement",
    ].flatMap((key) => [
      [key, enter],
      [`${key}:exit`, exit],
    ]),
  );
}

module.exports = {
  bindingScope,
  clearNullableBinding,
  createScope,
  functionScopeVisitors,
  isNullableBinding,
  lexicalScopeVisitors,
  objectProps,
  variableScope,
};
