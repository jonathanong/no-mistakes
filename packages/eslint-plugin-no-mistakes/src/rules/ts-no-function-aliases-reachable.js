"use strict";

const { resolveVariable } = require("./ts-no-function-aliases-self");
const { staticMemberName } = require("./ts-no-function-aliases-ast");

function moduleBinding(variable) {
  return variable?.scope.block.type === "Program";
}

function addExport(declarations, variable) {
  if (!moduleBinding(variable)) return;
  for (const def of variable.defs) {
    if (def.node.type === "FunctionDeclaration" || def.parent?.kind === "const")
      declarations.add(def.node);
  }
}

function exportedDeclarations(program, sourceCode) {
  const declarations = new Set();
  for (const statement of program.body) {
    if (
      statement.type !== "ExportNamedDeclaration" &&
      statement.type !== "ExportDefaultDeclaration"
    )
      continue;
    const declaration = statement.declaration;
    if (declaration?.type === "FunctionDeclaration") declarations.add(declaration);
    if (declaration?.type === "Identifier") {
      const variable = resolveVariable(sourceCode, declaration);
      // `export default name` snapshots a value; a later write can make the
      // callable used by this file unavailable through that default export.
      if (!variable?.references.some((reference) => reference.isWrite() && !reference.init))
        addExport(declarations, variable);
    }
    if (declaration?.type === "VariableDeclaration" && declaration.kind === "const") {
      for (const item of declaration.declarations) declarations.add(item);
    }
    if (statement.source || statement.exportKind === "type") continue;
    for (const specifier of statement.specifiers || []) {
      if (specifier.type !== "ExportSpecifier" || specifier.exportKind === "type") continue;
      addExport(declarations, resolveVariable(sourceCode, specifier.local));
    }
  }
  return declarations;
}

function importedPath(target, sourceCode) {
  let root = target;
  while (root.type === "MemberExpression" && staticMemberName(root) !== null) {
    root = root.object;
  }
  // Do not unwrap assertions or call results on receivers: they can establish
  // a different type or a private runtime object instead of the imported API.
  if (root.type !== "Identifier") return false;
  const variable = resolveVariable(sourceCode, root);
  const def = variable?.defs.length === 1 ? variable.defs[0] : null;
  return (
    def?.type === "ImportBinding" &&
    def.parent.importKind !== "type" &&
    def.node.importKind !== "type"
  );
}

function reachableTarget(target, sourceCode, declarations) {
  if (importedPath(target, sourceCode)) return { imported: true };
  if (target.type !== "Identifier") return null;
  const variable = resolveVariable(sourceCode, target);
  if (!moduleBinding(variable)) return null;
  if (variable.defs.length !== 1) return null;
  const declaration = variable.defs[0].node;
  if (!declarations.has(declaration)) return null;
  const callable = declaration.type === "FunctionDeclaration" ? declaration : declaration.init;
  return { callable, annotation: declaration.id?.typeAnnotation };
}

module.exports = { exportedDeclarations, reachableTarget };
