"use strict";

const api = require("./no-object-mutation-runtime");
const { bindName } = require("./no-object-mutation-bind");
const { markEscaped } = require("./no-object-mutation-write");
const {
  child,
  createFresh,
  joinValues,
  lookup,
  remember,
  unknown,
} = require("./no-object-mutation-values");

function visitClassNode(state, node, scope, role) {
  const value = createFresh(scope);
  value.isClass = true;
  const prior = node.type === "ClassDeclaration" && node.id ? lookup(scope, node.id.name) : null;
  if (node.type === "ClassDeclaration" && node.id && !prior) {
    bindName(node.id.name, value, scope, "local");
  }
  const superTag = node.superClass ? api.visit(state, node.superClass, scope, "read") : null;
  copyMethods(value, superTag);
  const classEnv = child(scope);
  classEnv.inClass = true;
  if (node.type === "ClassExpression" && node.id) {
    bindName(node.id.name, value, classEnv, "local");
  }
  const decorated = visitMembers(state, node, classEnv, value);
  // A module class is already shared. Escaping it here would walk methods
  // before later module bindings exist; flush walks those methods instead.
  if (decorated || role === "escape") {
    markEscaped(state, value);
  }
  if (prior && prior.kind === "module") {
    return prior.tag;
  }
  return value;
}

function copyMethods(value, tag) {
  if (!tag || tag === value) {
    return;
  }
  if (tag.type === "union") {
    for (const member of tag.members) {
      copyMethods(value, member);
    }
    return;
  }
  if (tag.type !== "fresh" || !tag.isClass) {
    return;
  }
  for (const item of tag.items) {
    value.items.push(item);
  }
}

function visitMembers(state, node, classEnv, value) {
  let decorated = false;
  for (const decorator of node.decorators ?? []) {
    api.visit(state, decorator.expression, classEnv, "escape");
    decorated = true;
  }
  for (const element of node.body.body) {
    if (visitElement(state, element, classEnv, value)) {
      decorated = true;
    }
  }
  return decorated;
}

function visitElement(state, element, classEnv, value) {
  let decorated = false;
  if (element.computed && element.key) {
    api.visit(state, element.key, classEnv, "read");
  }
  for (const decorator of element.decorators ?? []) {
    api.visit(state, decorator.expression, classEnv, "escape");
    decorated = true;
  }
  if (element.type === "StaticBlock") {
    api.visit(state, element, classEnv, "read");
    return decorated;
  }
  if (!element.value) {
    return decorated;
  }
  // Instance fields run on construction, after the class value may have escaped.
  if (element.type === "PropertyDefinition" && !element.static) {
    value.items.push(api.deferExpression(state, element.value, classEnv));
    return decorated;
  }
  const visited = api.visit(state, element.value, classEnv, "read");
  if (element.type === "MethodDefinition" && visited.type === "fn") {
    visited.methodKind = element.kind;
    visited.staticMethod = Boolean(element.static);
    if (!element.computed && element.key?.type === "Identifier") {
      visited.methodName = element.key.name;
    }
    value.items.push(visited);
    return decorated;
  }
  remember(value, visited);
  return decorated;
}

function constructInstance(scope, tag) {
  if (!tag || tag.type === "unknown" || tag.type === "fn") {
    return unknown();
  }
  if (tag.type === "union") {
    let joined = unknown();
    for (const member of tag.members) {
      joined = joinValues(joined, constructInstance(scope, member));
    }
    return joined;
  }
  if (tag.type !== "fresh" || !tag.isClass) {
    return unknown();
  }
  const instance = createFresh(scope);
  for (const item of tag.items) {
    // Construction already ran the constructor and instance fields.
    // Static methods are not reachable from the instance.
    if (item.methodKind === "constructor" || item.methodKind === "field" || item.staticMethod) {
      continue;
    }
    instance.items.push(item);
  }
  return instance;
}

module.exports = { constructInstance, visitClassNode };
