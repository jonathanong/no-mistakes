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

function isValueShadow(declaration, name, type) {
  return (
    (declaration.type === "ClassDeclaration" || declaration.type === "TSEnumDeclaration") &&
    declaration.id?.name === name &&
    !containsNode(declaration, type)
  );
}

function nearestDeclarations(type, name) {
  let current = type?.parent;
  while (current) {
    const found = [];
    const shadows = [];
    const statements =
      current.body?.body && !containsNode(current.body, type)
        ? null
        : bodyStatements(current, type);
    for (const statement of statements ?? []) {
      const declaration = statement.declaration ?? statement;
      if (isValueShadow(declaration, name, type)) shadows.push(declaration);
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
    if (shadows.length) return shadows;
    if (current.type === "Program") return null;
    current = current.parent;
  }
  return null;
}

function enclosingTypeParameter(type, name, local) {
  let current = type?.parent;
  while (current && current.type !== "Program") {
    const param = current.typeParameters?.params?.find((item) => item.name?.name === name);
    if (
      param &&
      !local?.some((item) => item.range[0] > current.range[0] && item.range[1] < current.range[1])
    )
      return param;
    current = current.parent;
  }
  return null;
}

module.exports = { enclosingTypeParameter, nearestDeclarations };
