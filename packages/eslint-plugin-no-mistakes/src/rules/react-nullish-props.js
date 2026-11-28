"use strict";

const { literalString } = require("./module-mock-helpers");
const { unwrapSequence } = require("./async-ast");
const {
  assertedType,
  keyName,
  reactTypeStatus,
  typeAnnotation,
  typeName,
} = require("../react-node-types");

function containsNode(root, node) {
  for (let current = node; current; current = current.parent) if (current === root) return true;
  return false;
}

function nearestNestedType(type, name) {
  let current = type?.parent;
  while (current && current.type !== "Program") {
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
      if (
        typed &&
        declaration.id?.name === name &&
        !containsNode(declaration, type) &&
        containsNode(declaration.parent, type)
      ) {
        return declaration;
      }
    }
    current = current.parent;
  }
  return null;
}

function inheritedFacts(heritage, facts, seen) {
  const expression = heritage?.expression;
  if (expression?.type !== "Identifier") return null;
  const nested = nearestNestedType(heritage, expression.name);
  if (!nested)
    return {
      props: facts.objectProps.get(expression.name) || null,
      unresolved: facts.unresolvedObjectProps?.get(expression.name) || null,
    };
  if (nested.type === "TSTypeAliasDeclaration")
    return classifiedProps(nested.typeAnnotation, facts, seen);
  return classifiedProps(
    { type: "TSTypeReference", typeName: expression, parent: heritage },
    facts,
    seen,
  );
}

function classifiedProps(type, facts, seen = new Set()) {
  if (type && type.type === "TSTypeLiteral") {
    const props = new Set();
    const unresolved = new Set();
    const declared = new Set();
    for (const member of type.members) {
      if (member.type !== "TSPropertySignature") continue;
      const name = member.computed ? literalString(member.key) : keyName(member.key);
      if (!name) continue;
      declared.add(name);
      const status = reactTypeStatus(typeAnnotation(member), facts);
      if (status === "yes") props.add(name);
      else if (status === "unresolved") unresolved.add(name);
    }
    return { props, unresolved, declared };
  }
  const name = typeName(type);
  const local = name && !name.includes(".") ? nearestNestedType(type, name) : null;
  if (local && seen.has(local)) return { props: null, unresolved: null };
  if (local?.type === "TSTypeAliasDeclaration") {
    seen.add(local);
    return classifiedProps(local.typeAnnotation, facts, seen);
  }
  if (local?.type === "TSInterfaceDeclaration") {
    seen.add(local);
    const own = classifiedProps(
      { type: "TSTypeLiteral", members: local.body?.body ?? [] },
      facts,
      seen,
    );
    const props = new Set(own.props);
    const unresolved = new Set(own.unresolved);
    for (const heritage of local.extends || []) {
      const inherited = inheritedFacts(heritage, facts, seen);
      for (const key of inherited?.props || []) if (!own.declared.has(key)) props.add(key);
      for (const key of inherited?.unresolved || [])
        if (!own.declared.has(key) && !props.has(key)) unresolved.add(key);
    }
    return { props, unresolved, declared: own.declared };
  }
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
  const key = member.computed ? literalString(member.property) : keyName(member.property);
  const asserted = assertedObjectFacts(member.object, facts);
  if (asserted) {
    if (key && asserted.props?.has(key)) return "yes";
    if (key && asserted.unresolved?.has(key)) return "unresolved";
    return "no";
  }
  const object = unwrapSequence(member.object);
  if (object?.type !== "Identifier" || !key) return "no";
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
