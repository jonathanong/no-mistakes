"use strict";

const api = require("./no-object-mutation-runtime");
const { assignPattern, bindingEscapes } = require("./no-object-mutation-bind");
const { readOnlyFresh } = require("./no-object-mutation-join");
const { considerWrite, markEscaped } = require("./no-object-mutation-write");
const { literalKey, memberKey, staticKey } = require("./no-object-mutation-syntax");
const { createFresh, remember, unknown } = require("./no-object-mutation-values");

const SKIP_KEYS = new Set([
  "decorators",
  "end",
  "loc",
  "parent",
  "range",
  "returnType",
  "start",
  "tokens",
  "typeAnnotation",
  "typeParameters",
]);

function visitAssignment(state, node, scope) {
  if (node.left.type === "MemberExpression") {
    const target = api.visit(state, node.left.object, scope, "read");
    const rhs = api.visit(state, node.right, scope, readOnlyFresh(target) ? "read" : "escape");
    considerWrite(state, target, node, {
      viaCall: false,
      objectNode: node.left,
      key: memberKey(node.left),
      rhs,
      scope,
    });
    return rhs;
  }
  const escapes = node.left.type === "Identifier" && bindingEscapes(scope, node.left.name);
  const rhs = api.visit(state, node.right, scope, escapes ? "escape" : "read");
  assignPattern(state, node.left, rhs, scope);
  return rhs;
}

function visitObject(state, node, scope, role) {
  const fresh = createFresh(scope);
  for (const property of node.properties) {
    if (property.type === "SpreadElement") {
      remember(fresh, api.visit(state, property.argument, scope, "read"));
      continue;
    }
    if (property.computed) {
      api.visit(state, property.key, scope, "read");
    }
    const value = api.visit(state, property.value, scope, "read");
    if (property.kind !== "init" || property.method) {
      // Methods are not data fields, but they escape with the object.
      fresh.items.push(value);
      remember(fresh, unknown());
      continue;
    }
    const key = property.computed ? literalKey(property.key) : staticKey(property.key);
    if (key) {
      fresh.fields.set(key, value);
    } else {
      remember(fresh, value);
    }
  }
  if (role === "escape") {
    markEscaped(state, fresh);
  }
  return fresh;
}

function visitArray(state, node, scope, role) {
  const fresh = createFresh(scope);
  for (const element of node.elements) {
    if (!element) {
      continue;
    }
    remember(
      fresh,
      element.type === "SpreadElement"
        ? api.visit(state, element.argument, scope, "read")
        : api.visit(state, element, scope, "read"),
    );
  }
  if (role === "escape") {
    markEscaped(state, fresh);
  }
  return fresh;
}

function walkChildren(state, node, scope) {
  for (const key of Object.keys(node)) {
    if (SKIP_KEYS.has(key)) {
      continue;
    }
    const value = node[key];
    if (Array.isArray(value)) {
      for (const childNode of value) {
        if (!childNode) {
          continue;
        }
        if (typeof childNode.type === "string") {
          api.visit(state, childNode, scope, "read");
        }
      }
    } else if (value && typeof value.type === "string") {
      api.visit(state, value, scope, "read");
    } else {
      continue;
    }
  }
}

module.exports = {
  visitArray,
  visitAssignment,
  visitObject,
  walkChildren,
};
