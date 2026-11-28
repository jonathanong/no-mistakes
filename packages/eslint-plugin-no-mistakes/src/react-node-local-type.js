"use strict";

function containsNode(root, node) {
  for (let current = node; current; current = current.parent) if (current === root) return true;
  return false;
}

function parameterShadows(declaration, name) {
  return declaration.typeParameters?.params?.some((param) => param.name?.name === name) === true;
}

function caseStatements(current, node) {
  if (current?.type !== "SwitchStatement" || containsNode(current.discriminant, node)) return null;
  const cases = current.cases ?? [];
  if (!cases.some((item) => item.consequent?.some((statement) => containsNode(statement, node))))
    return null;
  const statements = [];
  for (const item of cases) statements.push(...(item.consequent ?? []));
  return statements;
}

function aliasScope(declaration) {
  if (declaration.parent?.type === "SwitchCase") return declaration.parent.parent;
  return declaration.parent?.type === "ExportNamedDeclaration"
    ? declaration.parent.parent
    : declaration.parent;
}

function defaultedAliasStatus(declaration, facts, seen, reactTypeStatus) {
  const params = declaration.typeParameters?.params ?? [];
  if (!params.some((param) => param.default)) return null;
  const body = declaration.typeAnnotation;
  const bodyName =
    body?.type === "TSTypeReference" && body.typeName?.type === "Identifier"
      ? body.typeName.name
      : null;
  const param = params.find((item) => item.name?.name === bodyName);
  if (!param) return null;
  const next = new Set(seen);
  next.add(declaration);
  return param.default ? reactTypeStatus(param.default, facts, next) : "unresolved";
}

function localTypeStatus(node, name, facts, seen, reactTypeStatus) {
  let current = node?.parent;
  while (current) {
    if (parameterShadows(current, name)) {
      const param = current.typeParameters.params.find((item) => item.name?.name === name);
      if (seen.has(param)) return "unresolved";
      const next = new Set(seen);
      next.add(param);
      if (param?.constraint) return reactTypeStatus(param.constraint, facts, next);
      const known =
        facts.reactNodeNames.has(name) ||
        facts.aliases.get(name) === true ||
        Boolean(facts.unresolvedTypes?.has(name));
      return known ? "unresolved" : "no";
    }
    const statements = Array.isArray(current.body)
      ? current.body
      : Array.isArray(current.body?.body)
        ? current.body.body
        : caseStatements(current, node);
    for (const statement of statements ?? []) {
      const declaration = statement.declaration ?? statement;
      const scope = aliasScope(declaration);
      if (
        (declaration.type === "ClassDeclaration" || declaration.type === "TSEnumDeclaration") &&
        declaration.id?.name === name &&
        containsNode(scope, node)
      )
        return "no";
      const typed =
        declaration.type === "TSTypeAliasDeclaration" ||
        declaration.type === "TSInterfaceDeclaration";
      const selfAlias =
        declaration.type === "TSTypeAliasDeclaration" && !parameterShadows(declaration, name);
      if (
        !typed ||
        declaration.id?.name !== name ||
        (containsNode(declaration, node) &&
          !selfAlias &&
          declaration.type !== "TSInterfaceDeclaration")
      )
        continue;
      if (!containsNode(scope, node)) continue;
      if (declaration.type === "TSInterfaceDeclaration") return "no";
      if (seen.has(declaration)) return "unresolved";
      const supplied = node.typeArguments?.params || node.typeParameters?.params;
      const params = declaration.typeParameters?.params ?? [];
      if (params.length && supplied?.length) return "unresolved";
      const defaulted = defaultedAliasStatus(declaration, facts, seen, reactTypeStatus);
      if (defaulted) return defaulted;
      const next = new Set(seen);
      next.add(declaration);
      return reactTypeStatus(declaration.typeAnnotation, facts, next);
    }
    current = current.parent;
  }
  return null;
}

module.exports = { localTypeStatus };
