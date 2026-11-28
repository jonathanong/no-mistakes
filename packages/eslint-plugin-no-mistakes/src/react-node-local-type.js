"use strict";

function containsNode(root, node) {
  for (let current = node; current; current = current.parent) if (current === root) return true;
  return false;
}

function parameterShadows(declaration, name) {
  return declaration.typeParameters?.params?.some((param) => param.name?.name === name) === true;
}

function localTypeStatus(node, name, facts, seen, reactTypeStatus) {
  let current = node?.parent;
  while (current) {
    const statements = Array.isArray(current.body)
      ? current.body
      : Array.isArray(current.body?.body)
        ? current.body.body
        : null;
    for (const statement of statements ?? []) {
      const declaration = statement.declaration ?? statement;
      const typed =
        declaration.type === "TSTypeAliasDeclaration" ||
        declaration.type === "TSInterfaceDeclaration";
      const selfAlias =
        declaration.type === "TSTypeAliasDeclaration" && !parameterShadows(declaration, name);
      if (
        !typed ||
        declaration.id?.name !== name ||
        (containsNode(declaration, node) && !selfAlias)
      )
        continue;
      const scope =
        declaration.parent?.type === "ExportNamedDeclaration"
          ? declaration.parent.parent
          : declaration.parent;
      if (!containsNode(scope, node)) continue;
      if (declaration.type === "TSInterfaceDeclaration") return "no";
      if (seen.has(declaration)) return "unresolved";
      const next = new Set(seen);
      next.add(declaration);
      return reactTypeStatus(declaration.typeAnnotation, facts, next);
    }
    current = current.parent;
  }
  return null;
}

module.exports = { localTypeStatus };
