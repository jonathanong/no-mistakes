"use strict";

const api = require("./no-object-mutation-runtime");
const { bindName, bindPattern } = require("./no-object-mutation-bind");
const { registerFn } = require("./no-object-mutation-fn");
const { seedHoisted } = require("./no-object-mutation-syntax");
const { child, createFresh, lookup, shared, unknown } = require("./no-object-mutation-values");

function visitNamespace(state, node, scope) {
  const sharing = sharesNamespace(node, scope);
  const name = namespaceRoot(node.id);
  if (name) {
    bindNamespace(name, scope, sharing);
  }
  const bodyScope = child(scope);
  bodyScope.namespaceShared = sharing;
  walkNamespace(state, node.body, bodyScope, sharing);
  return unknown();
}

function namespaceRoot(id) {
  let current = id;
  while (current && current.type === "TSQualifiedName") {
    current = current.left;
  }
  if (current && current.type === "Identifier") {
    return current.name;
  }
  return null;
}

function sharesNamespace(node, scope) {
  // `declare module "fs"` is ambient. `namespace A.B` still follows the scope.
  if (node.id && node.id.type === "Literal") {
    return true;
  }
  return Boolean(scope.isModule || scope.namespaceShared);
}

function bindNamespace(name, scope, sharing) {
  if (lookup(scope, name)) {
    return;
  }
  bindName(name, sharing ? shared("module") : createFresh(scope), scope, "local");
}

function walkNamespace(state, body, scope, sharing) {
  // `namespace A.B` is a qualified name, and the body is a module block.
  seedHoisted(body.body, scope, false, registerFn);
  for (const statement of body.body) {
    if (sharing && statement.type === "ExportNamedDeclaration") {
      visitSharedExport(state, statement, scope);
    } else {
      api.visit(state, statement, scope, "read");
    }
  }
}

function visitSharedExport(state, statement, scope) {
  const declaration = statement.declaration;
  if (declaration && declaration.type === "TSModuleDeclaration") {
    visitNamespace(state, declaration, scope);
    return;
  }
  if (declaration && declaration.type === "VariableDeclaration") {
    for (const declarator of declaration.declarations) {
      if (declarator.init) {
        api.visit(state, declarator.init, scope, "escape");
      }
      bindPattern(state, declarator.id, shared("module"), scope, "local");
    }
  } else if (declaration) {
    api.visit(state, declaration, scope, "escape");
    if (declaration.id && declaration.id.name) {
      bindName(declaration.id.name, shared("module"), scope, "local");
    }
  }
  for (const specifier of statement.specifiers) {
    api.releaseEscaping(state, lookup(scope, specifier.local.name)?.tag);
  }
}

module.exports = { visitNamespace };
