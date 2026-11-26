"use strict";

const api = require("./no-object-mutation-runtime");
const { joinValues } = require("./no-object-mutation-join");
const { arrayParameter } = require("./no-object-mutation-receiver");
const { considerWrite } = require("./no-object-mutation-write");
const { literalKey, memberKey, staticKey, unwrap } = require("./no-object-mutation-syntax");
const {
  elementTag,
  freshCopy,
  lookup,
  readMember,
  shared,
} = require("./no-object-mutation-values");

function moduleTag(tag) {
  if (tag?.type === "mapset") {
    // The map object stays mutable. Values read back out of it are shared.
    tag.contents = shared("module");
    return tag;
  }
  const result = shared("module");
  if (tag?.intrinsic) result.intrinsic = tag.intrinsic;
  if (tag?.arraySubclass) result.arraySubclass = true;
  if (tag?.mutator) result.mutator = { ...tag.mutator, member: shared("module") };
  return result;
}

function bindName(name, tag, scope, kind) {
  const target = kind === "var" ? scope.functionEnv : scope;
  if (target.isModule) {
    // Escape after the module body so method walks see later bindings.
    if (tag?.type === "fresh") {
      target.moduleFresh.push(tag);
    }
    target.cells.set(name, { kind: "module", tag: moduleTag(tag) });
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

function unwrapParens(node) {
  let current = node;
  while (current?.type === "ParenthesizedExpression") current = current.expression;
  return current;
}

function assignBinding(state, name, tag, scope, join = false) {
  const owner = bindingScope(scope, name);
  const cell = owner?.cells.get(name);
  if (!cell || cell.kind === "import") {
    return;
  }
  if (bindingEscapes(scope, name)) {
    api.releaseEscaping(state, tag);
  }
  if (cell.kind === "module") {
    cell.tag = moduleTag(tag);
    return;
  }
  cell.tag = join ? joinValues(cell.tag, tag) : tag;
  cell.arrayParameter = false;
}

function assignPattern(state, pattern, tag, scope) {
  const target = unwrap(pattern);
  if (target.type === "MemberExpression") {
    if (target.computed) api.visit(state, target.property, scope, "read");
    considerWrite(state, api.visit(state, target.object, scope, "read"), target, {
      viaCall: false,
      objectNode: target,
      key: memberKey(target),
      rhs: tag,
      scope,
    });
    return;
  }
  if (target.type === "Identifier") {
    assignBinding(state, target.name, tag, scope, unwrapParens(pattern) !== target);
    return;
  }
  if (target.type === "AssignmentPattern") {
    api.visit(state, target.right, scope, "read");
    assignPattern(state, target.left, tag, scope);
    return;
  }
  if (target.type === "RestElement") {
    assignPattern(state, target.argument, freshCopy(tag, scope), scope);
    return;
  }
  if (target.type === "ObjectPattern") {
    for (const property of target.properties) {
      if (property.type === "RestElement") {
        assignPattern(state, property.argument, freshCopy(tag, scope), scope);
        continue;
      }
      const key = property.computed ? literalKey(property.key) : staticKey(property.key);
      if (property.computed) api.visit(state, property.key, scope, "read");
      assignPattern(state, property.value, key ? readMember(tag, key) : tag, scope);
    }
    return;
  }
  if (target.type === "ArrayPattern") {
    for (const element of target.elements) {
      if (element === null) continue;
      assignPattern(state, element, elementTag(tag), scope);
    }
  }
}

function bindPattern(state, pattern, tag, scope, kind, objectNode) {
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
    if (kind === "param") {
      lookup(scope, pattern.name).arrayParameter = arrayParameter(
        state.context.sourceCode,
        pattern,
      );
    }
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
      const field =
        kind === "param"
          ? shared("parameter")
          : key
            ? readMember(tag, key, property.value.type === "Identifier" ? objectNode : null)
            : tag;
      bindPattern(state, property.value, field, scope, kind);
    }
    return;
  }
  for (const element of pattern.elements) {
    // The else is required. An if/continue here makes v8 record a negative branch.
    if (element === null) {
      continue;
    } else {
      const next = kind === "param" ? shared("parameter") : elementTag(tag);
      bindPattern(state, element, next, scope, kind);
    }
  }
}

module.exports = { assignPattern, bindingEscapes, bindName, bindPattern };
