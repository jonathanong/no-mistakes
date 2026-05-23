"use strict";

function keyName(node) {
  if (!node) return null;
  if (node.type === "Identifier") return node.name;
  return node.type === "Literal" ? String(node.value) : null;
}

function unwrapType(node) {
  return node &&
    (node.type === "TSParenthesizedType" ||
      node.type === "TSOptionalType" ||
      node.type === "TSRestType")
    ? unwrapType(node.typeAnnotation)
    : node;
}

function typeName(node) {
  const current = unwrapType(node);
  if (!current) return null;
  if (current.type !== "TSTypeReference") return null;
  const name = current.typeName;
  if (name.type === "Identifier") return name.name;
  return name.type === "TSQualifiedName" &&
    name.left.type === "Identifier" &&
    name.right.type === "Identifier"
    ? `${name.left.name}.${name.right.name}`
    : null;
}

function typeAnnotation(node) {
  return node && node.typeAnnotation ? unwrapType(node.typeAnnotation.typeAnnotation) : null;
}

function createReactNodeFacts(program) {
  const reactNodeNames = new Set(["React.ReactNode"]);
  const aliases = new Map();
  const objectProps = new Map();

  function isReactNodeType(type) {
    const name = typeName(type);
    return Boolean(name && (reactNodeNames.has(name) || aliases.get(name) === true));
  }

  function collectMembers(members) {
    const props = new Set();
    for (const member of members || []) {
      if (member.type !== "TSPropertySignature" || !isReactNodeType(typeAnnotation(member))) {
        continue;
      }
      const name = keyName(member.key);
      if (name) props.add(name);
    }
    return props;
  }

  for (const statement of program.body || []) {
    if (statement.type !== "ImportDeclaration" || statement.source.value !== "react") continue;
    for (const specifier of statement.specifiers || []) {
      if (specifier.type === "ImportSpecifier" && keyName(specifier.imported) === "ReactNode") {
        reactNodeNames.add(specifier.local.name);
      }
    }
  }

  let changed = true;
  while (changed) {
    changed = false;
    for (const statement of program.body || []) {
      if (statement.type !== "TSTypeAliasDeclaration") continue;
      const name = statement.id.name;
      if (!aliases.has(name) && isReactNodeType(statement.typeAnnotation)) {
        aliases.set(name, true);
        changed = true;
      }
    }
  }

  for (const statement of program.body || []) {
    if (statement.type === "TSInterfaceDeclaration") {
      objectProps.set(statement.id.name, collectMembers(statement.body && statement.body.body));
    }
    if (
      statement.type === "TSTypeAliasDeclaration" &&
      statement.typeAnnotation.type === "TSTypeLiteral"
    ) {
      objectProps.set(statement.id.name, collectMembers(statement.typeAnnotation.members));
    }
  }

  return { aliases, objectProps, reactNodeNames };
}

module.exports = {
  createReactNodeFacts,
  keyName,
  typeAnnotation,
  typeName,
};
