"use strict";

const { localTypeStatus } = require("./react-node-local-type");
const { literalString } = require("./rules/module-mock-helpers");
const { constStringKey } = require("./rules/react-use-callee");
const { resolveVariable } = require("./rules/async-target-bindings");

const TYPE_SHADOW = new Set(["ClassName", "FunctionName", "TSModuleName", "Type", "TypeParameter"]);

function keyName(node) {
  if (!node) return null;
  if (node.type === "Identifier") return node.name;
  return node.type === "Literal" ? String(node.value) : null;
}

const UNWRAP_TYPES = new Set(["TSParenthesizedType", "TSOptionalType", "TSRestType"]);

function unwrapType(node) {
  return UNWRAP_TYPES.has(node?.type) ? unwrapType(node.typeAnnotation) : node;
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
  return current.type === "TSTypeReference" ? qualifierText(current.typeName) : null;
}

function typeAnnotation(node) {
  return node && node.typeAnnotation ? unwrapType(node.typeAnnotation.typeAnnotation) : null;
}

const TYPE_PRESERVING = new Set([
  "ChainExpression",
  "ParenthesizedExpression",
  "TSNonNullExpression",
  "TSSatisfiesExpression",
  "TSInstantiationExpression",
  "TypeCastExpression",
]);

function assertedType(node) {
  let current = node;
  while (current) {
    if (current.type === "TSAsExpression" || current.type === "TSTypeAssertion") {
      const type = current.typeAnnotation;
      const constAssertion =
        type?.type === "TSTypeReference" &&
        type.typeName?.type === "Identifier" &&
        type.typeName.name === "const";
      if (constAssertion) {
        current = current.expression;
        continue;
      }
      return type ?? null;
    }
    if (TYPE_PRESERVING.has(current.type)) {
      current = current.expression;
      continue;
    }
    if (current.type === "SequenceExpression") {
      current = current.expressions.at(-1);
      continue;
    }
    return null;
  }
  return null;
}

function qualifierShadowsImport(typeNode, facts) {
  const root = typeNode?.typeName?.type === "TSQualifiedName" ? typeNode.typeName.left : null;
  if (root?.type !== "Identifier" || !facts.context) return false;
  const variable = resolveVariable(root, facts.context);
  return Boolean(variable?.defs.length) && variable.defs.every((def) => TYPE_SHADOW.has(def.type));
}

function reactTypeStatus(type, facts, seen = new Set()) {
  const current = unwrapType(type);
  if (!current || !facts) return "no";
  if (current.type === "TSUnionType") {
    let unresolved = false;
    for (const part of current.types) {
      const status = reactTypeStatus(part, facts, seen);
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
  if (name && current.type === "TSTypeReference" && !name.includes(".")) {
    const local = localTypeStatus(current, name, facts, seen, reactTypeStatus);
    if (local !== null) return local;
  }
  if (qualifierShadowsImport(current, facts)) return "no";
  const root = name?.split(".")[0];
  if (root && name.endsWith(".ReactNode") && facts.unresolvedNamespaces?.has(root)) {
    return "unresolved";
  }
  if (name && (facts.reactNodeNames.has(name) || facts.aliases.get(name) === true)) return "yes";
  if (name && facts.unresolvedTypes?.has(name)) return "unresolved";
  return "no";
}

function bindPatternNames(pattern, defineBinding) {
  if (!pattern) return;
  if (pattern.type === "AssignmentPattern" || pattern.type === "RestElement") {
    bindPatternNames(pattern.left ?? pattern.argument, defineBinding);
    return;
  }
  if (pattern.type === "Identifier") {
    defineBinding(pattern.name);
    return;
  }
  if (pattern.type === "ArrayPattern") {
    for (const element of pattern.elements) bindPatternNames(element, defineBinding);
    return;
  }
  if (pattern.type !== "ObjectPattern") return;
  for (const property of pattern.properties) {
    bindPatternNames(
      property.type === "RestElement" ? property.argument : property.value,
      defineBinding,
    );
  }
}

function definePattern(
  pattern,
  props,
  defineBinding,
  defineReactNode,
  unresolved,
  defineUnresolved,
  context,
) {
  if (!pattern || pattern.type !== "ObjectPattern") return;
  bindPatternNames(pattern, defineBinding);
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const name = property.computed
      ? literalString(property.key) || (context && constStringKey(property.key, context))
      : keyName(property.key);
    const value =
      property.value?.type === "AssignmentPattern" ? property.value.left : property.value;
    if (value?.type !== "Identifier") continue;
    if (name && props?.has(name)) defineReactNode(value.name);
    else if (name && unresolved?.has(name)) defineUnresolved?.(value.name);
    else if (!name && property.computed && (props?.size || unresolved?.size))
      defineUnresolved?.(value.name);
  }
}

module.exports = {
  assertedType,
  bindPatternNames,
  definePattern,
  keyName,
  qualifierText,
  reactTypeStatus,
  typeAnnotation,
  typeName,
};
