"use strict";

function containsNode(root, node) {
  for (let current = node; current; current = current.parent) if (current === root) return true;
  return false;
}

function bodyStatements(node, type) {
  if (Array.isArray(node?.body)) return node.body;
  if (Array.isArray(node?.body?.body)) return node.body.body;
  if (node?.type !== "SwitchStatement" || containsNode(node.discriminant, type)) return null;
  const cases = node.cases ?? [];
  const inside = cases.some((item) =>
    item.consequent?.some((statement) => containsNode(statement, type)),
  );
  if (!inside) return null;
  const statements = [];
  for (const item of cases) statements.push(...(item.consequent ?? []));
  return statements;
}

function nearestDeclarations(type, name) {
  let current = type?.parent;
  while (current && current.type !== "Program") {
    const found = [];
    for (const statement of bodyStatements(current, type) ?? []) {
      const declaration = statement.declaration ?? statement;
      const typed =
        declaration.type === "TSTypeAliasDeclaration" ||
        declaration.type === "TSInterfaceDeclaration";
      const contained = containsNode(declaration, type);
      const selfAlias =
        declaration.type === "TSTypeAliasDeclaration" &&
        contained &&
        !declaration.typeParameters?.params?.some((param) => param.name?.name === name);
      if (typed && declaration.id?.name === name && (!contained || selfAlias))
        found.push(declaration);
    }
    if (found.length) return found;
    current = current.parent;
  }
  return null;
}

module.exports = { nearestDeclarations };
