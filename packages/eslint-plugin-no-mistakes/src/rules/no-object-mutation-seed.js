"use strict";

const { callable, lookup, shared } = require("./no-object-mutation-values");

function functionStatements(node) {
  if (node.body && node.body.type === "BlockStatement") {
    return node.body.body;
  }
  return node.body;
}

function hoistedDeclaration(statement) {
  if (statement.type === "FunctionDeclaration") {
    return statement;
  }
  if (
    statement.type === "ExportNamedDeclaration" &&
    statement.declaration &&
    statement.declaration.type === "FunctionDeclaration"
  ) {
    return statement.declaration;
  }
  return null;
}

function classDeclaration(statement) {
  if (statement.type === "ClassDeclaration") {
    return statement;
  }
  if (
    statement.type === "ExportNamedDeclaration" &&
    statement.declaration?.type === "ClassDeclaration"
  ) {
    return statement.declaration;
  }
  return null;
}

function hoistedFn(node, scope) {
  if (node.id) {
    return callable(lookup(scope, node.id.name)?.tag);
  }
  return null;
}

function seedImports(program, scope) {
  for (const statement of program.body) {
    if (statement.type !== "ImportDeclaration" || statement.importKind === "type") {
      continue;
    }
    for (const specifier of statement.specifiers) {
      if (specifier.type === "ImportSpecifier" && specifier.importKind === "type") {
        continue;
      }
      scope.cells.set(specifier.local.name, { kind: "import", tag: shared("import") });
    }
  }
}

function seedHoisted(statements, scope, moduleLevel, register) {
  for (const statement of statements) {
    const decl = hoistedDeclaration(statement);
    if (!decl) {
      continue;
    }
    const tag = {
      type: "fn",
      deferred: [],
      captured: [],
      didEscape: false,
      walked: false,
      registered: false,
      node: decl,
      scope,
    };
    register(tag);
    if (moduleLevel) {
      scope.cells.set(decl.id.name, {
        kind: "module",
        tag: { type: "shared", origin: "module", fn: tag },
      });
    } else {
      scope.cells.set(decl.id.name, { kind: "local", tag });
    }
  }
}

function seedClasses(statements, scope) {
  if (!scope.isModule) {
    return;
  }
  for (const statement of statements) {
    const decl = classDeclaration(statement);
    if (!decl) {
      continue;
    }
    scope.cells.set(decl.id.name, { kind: "module", tag: shared("module") });
  }
}

module.exports = {
  classDeclaration,
  functionStatements,
  hoistedDeclaration,
  hoistedFn,
  seedClasses,
  seedHoisted,
  seedImports,
};
