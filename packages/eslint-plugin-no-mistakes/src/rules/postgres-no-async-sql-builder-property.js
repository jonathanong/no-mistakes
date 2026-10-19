"use strict";

const {
  FUNCTION_MEMBERS,
  NON_PROMISE_TYPES,
  annotationContract,
  combine,
  lookup,
  referenceName,
  staticKeyName,
  typeLiteralFlag,
  unwrapType,
} = require("./postgres-no-async-sql-builder-types");

function callableReturnContract(typeNode, types, seen = new Set()) {
  const type = unwrapType(typeNode);
  if (!type) return "unknown";
  if (type.type === "TSUnionType") {
    // Any Promise-returning alternative can be what callers invoke.
    return combine(type.types.map((part) => callableReturnContract(part, types, seen)));
  }
  if (type.type === "TSIntersectionType")
    return combine(type.types.map((part) => callableReturnContract(part, types, seen)));
  if (type.type === "TSFunctionType") return annotationContract(type.returnType, types);
  if (type.type === "TSTypeLiteral") {
    const call = type.members.find((member) => member.type === "TSCallSignatureDeclaration");
    if (call) return annotationContract(call.returnType, types);
    // A construct signature is not a value the caller can treat as SQL.
    // A literal with no call signature describes the function itself. Report
    // only when a required member is one a function does not have.
    if (type.members.some((member) => member.type === "TSConstructSignatureDeclaration"))
      return "unknown";
    return typeLiteralFlag(type, FUNCTION_MEMBERS) === "no" ? "allows" : "unknown";
  }
  if (type.type === "TSTypeReference") {
    const name = referenceName(type.typeName);
    if (!name || name === "Promise") return "unknown";
    const alias = lookup(types.aliases, name);
    if (!alias) return "unknown";
    if (seen.has(name)) return "unknown";
    seen.add(name);
    return callableReturnContract(alias, types, seen);
  }
  if (NON_PROMISE_TYPES.has(type.type)) return "allows";
  return "unknown";
}

function valueContract(typeNode, types) {
  return combine([annotationContract(typeNode, types), callableReturnContract(typeNode, types)]);
}

function membersContract(members, name, types) {
  const member = members.find((item) => staticKeyName(item) === name);
  if (!member) return "missing";
  if (member.type === "TSMethodSignature") return annotationContract(member.returnType, types);
  // A named type element that is not a method is a property. Other type
  // elements have no static name, so they cannot be selected above.
  return valueContract(member.typeAnnotation, types);
}

function interfacePropertyContract(face, name, types, seen) {
  if (seen.has(face)) return "unknown";
  seen.add(face);
  const own = membersContract(face.body.body, name, types);
  if (own === "requires" || own === "unknown") return own;
  let result = own === "allows" ? "allows" : "missing";
  for (const extend of face.extends || []) {
    const parent = lookup(types.interfaces, extend.expression?.name);
    if (!parent) return "unknown";
    const inherited = interfacePropertyContract(parent, name, types, seen);
    if (inherited === "requires" || inherited === "unknown") return inherited;
    if (inherited === "allows") result = "allows";
  }
  return result;
}

function typePropertyContract(typeNode, name, types, seen) {
  const type = unwrapType(typeNode);
  if (type.type === "TSTypeLiteral") return membersContract(type.members, name, types);
  if (type.type === "TSUnionType") {
    const parts = type.types.map((part) => typePropertyContract(part, name, types, seen));
    if (parts.some((part) => part === "unknown" || part === "missing")) return "unknown";
    return parts.every((part) => part === "requires") ? "requires" : "allows";
  }
  if (type.type === "TSIntersectionType") {
    let found = false;
    for (const part of type.types) {
      const contract = typePropertyContract(part, name, types, seen);
      if (contract === "requires" || contract === "unknown") return contract;
      if (contract === "allows") found = true;
    }
    return found ? "allows" : "missing";
  }
  if (type.type !== "TSTypeReference") return "unknown";
  const nameRef = referenceName(type.typeName);
  if (!nameRef || nameRef === "Promise" || seen.has(nameRef)) return "unknown";
  seen.add(nameRef);
  const alias = lookup(types.aliases, nameRef);
  if (alias) return typePropertyContract(alias, name, types, seen);
  const face = lookup(types.interfaces, nameRef);
  return face ? interfacePropertyContract(face, name, types, seen) : "unknown";
}

function knownPropertyContract(typeNode, name, types) {
  const contract = typePropertyContract(typeNode, name, types, new Set());
  return contract === "missing" ? "unknown" : contract;
}

function contextualTypeNode(object) {
  const parent = object.parent;
  if (
    parent?.type === "TSAsExpression" ||
    parent?.type === "TSSatisfiesExpression" ||
    parent?.type === "TSTypeAssertion"
  ) {
    return parent.typeAnnotation;
  }
  if (parent?.type === "VariableDeclarator") return parent.id.typeAnnotation || null;
  return null;
}

function contextualContract(func, types) {
  const property = func.parent;
  if (property?.type !== "Property" || property.parent?.type !== "ObjectExpression")
    return "allows";
  const typeNode = contextualTypeNode(property.parent);
  if (!typeNode) return "allows";
  const name = staticKeyName(property);
  return name ? knownPropertyContract(typeNode, name, types) : "unknown";
}

module.exports = {
  callableReturnContract,
  contextualContract,
  interfacePropertyContract,
  valueContract,
};
