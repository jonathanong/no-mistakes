"use strict";

const { literalString } = require("./rules/module-mock-helpers");

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

function qualifierText(node) {
  if (!node) return null;
  if (node.type === "Identifier") return node.name;
  return node.type === "TSQualifiedName" &&
    node.left.type === "Identifier" &&
    node.right.type === "Identifier"
    ? `${node.left.name}.${node.right.name}`
    : null;
}

function importTypeSpecifier(node) {
  if (node.source) return node.source;
  return node.argument?.type === "TSLiteralType" ? node.argument.literal : node.argument;
}

function typeName(node) {
  const current = unwrapType(node);
  if (!current) return null;
  if (current.type === "TSImportType") {
    const moduleName = literalString(importTypeSpecifier(current));
    return moduleName === "react" && qualifierText(current.qualifier) === "ReactNode"
      ? "React.ReactNode"
      : null;
  }
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

function reactTypeStatus(type, facts) {
  const current = unwrapType(type);
  if (!current || !facts) return "no";
  if (current.type === "TSUnionType") {
    let unresolved = false;
    for (const part of current.types) {
      const status = reactTypeStatus(part, facts);
      if (status === "yes") return "yes";
      if (status === "unresolved") unresolved = true;
    }
    return unresolved ? "unresolved" : "no";
  }
  if (current.type === "TSImportType") {
    const specifier = importTypeSpecifier(current);
    const moduleName = literalString(specifier);
    const qualifier = qualifierText(current.qualifier);
    if (moduleName === "react" && qualifier === "ReactNode") return "yes";
    if (moduleName === null && specifier && (!qualifier || qualifier === "ReactNode")) {
      return "unresolved";
    }
    return "no";
  }
  const name = typeName(current);
  if (name && (facts.reactNodeNames.has(name) || facts.aliases.get(name) === true)) return "yes";
  if (name && facts.unresolvedTypes?.has(name)) return "unresolved";
  const root = name?.split(".")[0];
  if (root && facts.unresolvedNamespaces?.has(root) && name.endsWith(".ReactNode")) {
    return "unresolved";
  }
  return "no";
}

function definePattern(
  pattern,
  props,
  defineBinding,
  defineReactNode,
  unresolved,
  defineUnresolved,
) {
  if (!pattern || pattern.type !== "ObjectPattern") return;
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const name = property.computed
      ? literalString(property.key) || keyName(property.key)
      : keyName(property.key);
    const value =
      property.value?.type === "AssignmentPattern" ? property.value.left : property.value;
    if (value?.type !== "Identifier") continue;
    defineBinding(value.name);
    if (name && props?.has(name)) defineReactNode(value.name);
    else if (name && unresolved?.has(name)) defineUnresolved?.(value.name);
  }
}

module.exports = {
  definePattern,
  keyName,
  qualifierText,
  reactTypeStatus,
  typeAnnotation,
  typeName,
};
