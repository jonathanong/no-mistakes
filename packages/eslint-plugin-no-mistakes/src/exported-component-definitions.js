"use strict";

const { functionFromExpression } = require("./component-functions");

function collectTopLevelDefinition(statement, definitions, opts, aliases, exported) {
  if (statement.type === "FunctionDeclaration" && statement.id) {
    definitions.set(statement.id.name, { name: statement.id.name, fn: statement });
    return;
  }
  if (statement.type !== "VariableDeclaration") {
    return;
  }
  for (const declaration of statement.declarations) {
    if (declaration.id.type !== "Identifier") {
      continue;
    }
    const fn = functionFromExpression(declaration.init, opts);
    if (fn) {
      definitions.set(declaration.id.name, { name: declaration.id.name, fn });
      continue;
    }
    if (declaration.init?.type === "Identifier" && declaration.parent?.kind === "const") {
      aliases.push({
        name: declaration.id.name,
        target: declaration.init.name,
        exported,
      });
    }
  }
}

module.exports = { collectTopLevelDefinition };
