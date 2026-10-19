"use strict";

const api = require("./no-object-mutation-runtime");
const { considerWrite } = require("./no-object-mutation-write");
const { literalKey, memberKey, staticKey } = require("./no-object-mutation-syntax");
const { elementTag, freshCopy, readMember, shared } = require("./no-object-mutation-values");

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

function bindingScope(scope, name) {
  for (let cursor = scope; cursor; cursor = cursor.outer) {
    if (cursor.cells.has(name)) {
      return cursor;
    }
  }
  return null;
}

function bindingEscapes(scope, name) {
  const owner = bindingScope(scope, name);
  if (!owner) {
    return false;
  }
  const cell = owner.cells.get(name);
  if (cell.kind === "import") {
    return false;
  }
  // A local `let` in this function can still be a builder. A cell in the
  // module or another function can be read after this write.
  return owner.isModule || owner.functionEnv !== scope.functionEnv;
}

function assignBinding(state, name, tag, scope) {
  const owner = bindingScope(scope, name);
  const cell = owner?.cells.get(name);
  if (!cell || cell.kind === "import") {
    return;
  }
  if (bindingEscapes(scope, name)) {
    api.releaseEscaping(state, tag);
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
    assignBinding(state, pattern.name, tag, scope);
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
      if (element === null) {
        continue;
      } else {
        assignPattern(state, element, elementTag(tag), scope);
      }
    }
  } else {
    return;
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
      if (element === null) {
        continue;
      } else {
        const next = kind === "param" ? shared("parameter") : elementTag(tag);
        bindPattern(state, element, next, scope, kind);
      }
    }
  } else {
    return;
  }
}

module.exports = {
  assignPattern,
  bindingEscapes,
  bindName,
  bindPattern,
};
