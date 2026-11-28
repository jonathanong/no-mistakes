"use strict";

const { unwrapSequence } = require("./async-ast");
const {
  assertedType,
  computedKey,
  keyName,
  reactTypeStatus,
  typeAnnotation,
  typeName,
} = require("../react-node-types");

function containsNode(root, node) {
  for (let current = node; current; current = current.parent) if (current === root) return true;
  return false;
}

function bodyStatements(node) {
  if (Array.isArray(node?.body)) return node.body;
  if (Array.isArray(node?.body?.body)) return node.body.body;
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

function nearestDeclarations(type, name) {
  let current = type?.parent;
  while (current && current.type !== "Program") {
    const found = [];
    for (const statement of bodyStatements(current) ?? []) {
      const declaration = statement.declaration ?? statement;
      const typed =
        declaration.type === "TSTypeAliasDeclaration" ||
        declaration.type === "TSInterfaceDeclaration";
      if (
        typed &&
        declaration.id?.name === name &&
        !containsNode(declaration, type) &&
        containsNode(declaration.parent, type)
      )
        found.push(declaration);
    }
    if (found.length) return found;
    current = current.parent;
  }
  return null;
}

function noteStatus(status, key, incoming) {
  const previous = status.get(key);
  status.set(key, previous && previous !== incoming ? "unresolved" : incoming);
}

function mergeInterfaces(declarations, facts, seen) {
  if (declarations.some((item) => seen.has(item))) return { props: null, unresolved: null };
  for (const item of declarations) seen.add(item);
  try {
    return mergeInterfaceBody(declarations, facts, seen);
  } finally {
    for (const item of declarations) seen.delete(item);
  }
}

function mergeInterfaceBody(declarations, facts, seen) {
  const status = new Map();
  const inheritedProps = new Set();
  const inheritedUnresolved = new Set();
  for (const declaration of declarations) {
    const own = classifiedProps(
      { type: "TSTypeLiteral", members: declaration.body?.body ?? [] },
      facts,
      seen,
    );
    for (const key of own.declared) {
      const incoming = own.props.has(key) ? "yes" : own.unresolved.has(key) ? "unresolved" : "no";
      noteStatus(status, key, incoming);
    }
    for (const heritage of declaration.extends || []) {
      const inherited = inheritedFacts(heritage, facts, seen);
      for (const key of inherited?.props || []) inheritedProps.add(key);
      for (const key of inherited?.unresolved || []) inheritedUnresolved.add(key);
    }
  }
  const props = new Set();
  const unresolved = new Set();
  for (const [key, value] of status) {
    if (value === "yes") props.add(key);
    else if (value === "unresolved") unresolved.add(key);
  }
  for (const key of inheritedProps) if (!status.has(key)) props.add(key);
  for (const key of inheritedUnresolved)
    if (!status.has(key) && !props.has(key)) unresolved.add(key);
  return { props, unresolved, declared: new Set(status.keys()) };
}

function inheritedFacts(heritage, facts, seen) {
  const expression = heritage?.expression;
  if (expression?.type !== "Identifier") return null;
  const nested = nearestDeclarations(heritage, expression.name);
  if (!nested)
    return {
      props: facts.objectProps.get(expression.name) || null,
      unresolved: facts.unresolvedObjectProps?.get(expression.name) || null,
    };
  if (nested.length === 1 && nested[0].type === "TSTypeAliasDeclaration")
    return classifiedProps(nested[0].typeAnnotation, facts, seen);
  if (nested.every((item) => item.type === "TSInterfaceDeclaration"))
    return mergeInterfaces(nested, facts, seen);
  return { props: null, unresolved: null };
}

function classifiedProps(type, facts, seen = new Set()) {
  if (type && type.type === "TSTypeLiteral") {
    const props = new Set();
    const unresolved = new Set();
    const declared = new Set();
    for (const member of type.members) {
      if (member.type !== "TSPropertySignature") continue;
      const name = member.computed ? computedKey(member.key) : keyName(member.key);
      if (name == null) continue;
      declared.add(name);
      const status = reactTypeStatus(typeAnnotation(member), facts);
      if (status === "yes") props.add(name);
      else if (status === "unresolved") unresolved.add(name);
    }
    return { props, unresolved, declared };
  }
  const name = typeName(type);
  const local = name && !name.includes(".") ? nearestDeclarations(type, name) : null;
  const parameter = name && !name.includes(".") ? enclosingTypeParameter(type, name, local) : null;
  if (local?.some((item) => seen.has(item)) && !parameter) return { props: null, unresolved: null };
  if (parameter) {
    if (seen.has(parameter)) return { props: null, unresolved: null };
    seen.add(parameter);
    try {
      return parameter.constraint
        ? classifiedProps(parameter.constraint, facts, seen)
        : { props: null, unresolved: null };
    } finally {
      seen.delete(parameter);
    }
  }
  if (local?.length === 1 && local[0].type === "TSTypeAliasDeclaration") {
    seen.add(local[0]);
    try {
      return classifiedProps(local[0].typeAnnotation, facts, seen);
    } finally {
      seen.delete(local[0]);
    }
  }
  if (local?.length && local.every((item) => item.type === "TSInterfaceDeclaration"))
    return mergeInterfaces(local, facts, seen);
  return {
    props: name ? facts.objectProps.get(name) || null : null,
    unresolved: name ? facts.unresolvedObjectProps?.get(name) || null : null,
  };
}

function assertedObjectFacts(node, facts) {
  const type = assertedType(node);
  return type ? classifiedProps(type, facts) : null;
}

function memberExpressionStatus(member, facts, lookup) {
  const key = member.computed ? computedKey(member.property) : keyName(member.property);
  const asserted = assertedObjectFacts(member.object, facts);
  if (asserted) {
    if (key != null && asserted.props?.has(key)) return "yes";
    if (key != null && asserted.unresolved?.has(key)) return "unresolved";
    return "no";
  }
  const object = unwrapSequence(member.object);
  if (object?.type !== "Identifier" || key == null) return "no";
  if (lookup(object, "objectTypes")?.has(key)) return "yes";
  if (lookup(object, "unresolvedMembers")?.has(key)) return "unresolved";
  return "no";
}

function objectInitFacts(node, facts, lookup) {
  const asserted = assertedObjectFacts(node, facts);
  if (asserted) return asserted;
  const init = unwrapSequence(node);
  if (init?.type !== "Identifier") return { props: null, unresolved: null };
  return { props: lookup(init, "objectTypes"), unresolved: lookup(init, "unresolvedMembers") };
}

function destructureMemberFacts(type, init, facts, lookup) {
  if (!type) return objectInitFacts(init, facts, lookup);
  const classified = classifiedProps(type, facts);
  if (classified.props || classified.unresolved) return classified;
  return reactTypeStatus(type, facts) === "unresolved"
    ? { props: null, unresolved: null, allUnresolved: true }
    : { props: null, unresolved: null };
}

module.exports = {
  assertedObjectFacts,
  classifiedProps,
  destructureMemberFacts,
  memberExpressionStatus,
  objectInitFacts,
};
