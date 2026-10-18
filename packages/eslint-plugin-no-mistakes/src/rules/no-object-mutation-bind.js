"use strict";

const api = require("./no-object-mutation-runtime");
const { considerWrite } = require("./no-object-mutation-write");
const { literalKey, memberKey, staticKey } = require("./no-object-mutation-syntax");
const {
  elementTag,
  freshCopy,
  lookup,
  readMember,
  shared,
} = require("./no-object-mutation-values");

function bindName(name, tag, scope, kind) {
  const target = kind === "var" ? scope.functionEnv : scope;
  if (target.isModule) {
    target.cells.set(name, {
      kind: "module",
      tag: tag?.type === "mapset" ? tag : shared("module"),
    });
    return;
  }
  target.cells.set(name, { kind, tag });
}

function assignBinding(name, tag, scope) {
  const cell = lookup(scope, name);
  if (!cell || cell.kind === "import") {
    return;
  }
  if (cell.kind === "module") {
    cell.tag = tag?.type === "mapset" ? tag : shared("module");
    return;
  }
  cell.tag = tag;
}

function assignPattern(state, pattern, tag, scope) {
  if (pattern.type === "MemberExpression") {
    considerWrite(state, api.visit(state, pattern.object, scope, "read"), pattern, {
      viaCall: false,
      objectNode: pattern,
      key: memberKey(pattern),
      rhs: tag,
      scope,
    });
    return;
  }
  if (pattern.type === "Identifier") {
    assignBinding(pattern.name, tag, scope);
    return;
  }
  if (pattern.type === "AssignmentPattern") {
    api.visit(state, pattern.right, scope, "read");
    assignPattern(state, pattern.left, tag, scope);
    return;
  }
  if (pattern.type === "RestElement") {
    assignPattern(state, pattern.argument, freshCopy(tag, scope), scope);
    return;
  }
  if (pattern.type === "ObjectPattern") {
    for (const property of pattern.properties) {
      if (property.type === "RestElement") {
        assignPattern(state, property.argument, freshCopy(tag, scope), scope);
        continue;
      }
      const key = property.computed ? literalKey(property.key) : staticKey(property.key);
      if (property.computed) {
        api.visit(state, property.key, scope, "read");
      }
      assignPattern(state, property.value, key ? readMember(tag, key) : tag, scope);
    }
    return;
  }
  if (pattern.type === "ArrayPattern") {
    for (const element of pattern.elements) {
      if (!element) {
        continue;
      }
      assignPattern(state, element, elementTag(tag), scope);
    }
  }
}

function bindPattern(state, pattern, tag, scope, kind) {
  if (pattern.type === "TSParameterProperty") {
    bindPattern(
      state,
      pattern.parameter,
      kind === "param" ? shared("parameter") : tag,
      scope,
      kind,
    );
    return;
  }
  if (pattern.type === "Identifier") {
    bindName(pattern.name, kind === "param" ? shared("parameter") : tag, scope, kind);
    return;
  }
  if (pattern.type === "AssignmentPattern") {
    api.visit(state, pattern.right, scope, "read");
    bindPattern(state, pattern.left, kind === "param" ? shared("parameter") : tag, scope, kind);
    return;
  }
  if (pattern.type === "RestElement") {
    const next = kind === "param" ? shared("parameter") : freshCopy(tag, scope);
    bindPattern(state, pattern.argument, next, scope, kind);
    return;
  }
  if (pattern.type === "ObjectPattern") {
    for (const property of pattern.properties) {
      if (property.type === "RestElement") {
        const next = kind === "param" ? shared("parameter") : freshCopy(tag, scope);
        bindPattern(state, property.argument, next, scope, kind);
        continue;
      }
      let key = null;
      if (property.computed) {
        api.visit(state, property.key, scope, "read");
        key = literalKey(property.key);
      } else {
        key = staticKey(property.key);
      }
      const field = kind === "param" ? shared("parameter") : key ? readMember(tag, key) : tag;
      bindPattern(state, property.value, field, scope, kind);
    }
    return;
  }
  if (pattern.type === "ArrayPattern") {
    for (const element of pattern.elements) {
      if (!element) {
        continue;
      }
      const next = kind === "param" ? shared("parameter") : elementTag(tag);
      bindPattern(state, element, next, scope, kind);
    }
  }
}

module.exports = {
  assignPattern,
  bindName,
  bindPattern,
};
