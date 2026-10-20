"use strict";

const {
  FUNCTION_MEMBERS,
  NON_PROMISE_TYPES,
  annotationContract,
  combine,
  lookup,
  referenceName,
  typeLiteralFlag,
  unwrapType,
} = require("./postgres-no-async-sql-builder-types");

// A non-callable literal reports only when a required member is neither a
// function member nor a Promise member. `then` alone does not prove that
// removing `async` leaves a working call.
const CALLABLE_LITERAL_MEMBERS = new Set([...FUNCTION_MEMBERS, "then", "catch", "finally"]);

function callableReturnContract(typeNode, types, seen = new Set()) {
  const type = unwrapType(typeNode);
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
    // only when a required member is neither a function member nor a Promise
    // member.
    if (type.members.some((member) => member.type === "TSConstructSignatureDeclaration"))
      return "unknown";
    return typeLiteralFlag(type, CALLABLE_LITERAL_MEMBERS) === "no" ? "allows" : "unknown";
  }
  if (type.type === "TSTypeReference") {
    const name = referenceName(type.typeName);
    if (!name || name === "Promise") return "unknown";
    const alias = lookup(types.aliases, name);
    if (alias) {
      if (seen.has(name)) return "unknown";
      seen.add(name);
      return callableReturnContract(alias, types, seen);
    }
    const face = lookup(types.interfaces, name);
    if (!face || seen.has(name)) return "unknown";
    seen.add(name);
    return interfaceCallableContract(face, types, seen);
  }
  if (NON_PROMISE_TYPES.has(type.type)) return "allows";
  return "unknown";
}

function interfaceCallableContract(face, types, seen) {
  if (typeLiteralFlag({ members: face.body.body }, CALLABLE_LITERAL_MEMBERS) === "no") {
    return "allows";
  }
  for (const extend of face.extends || []) {
    const parentName = extend.expression?.type === "Identifier" ? extend.expression.name : null;
    const parent = lookup(types.interfaces, parentName);
    if (!parent || seen.has(parentName)) return "unknown";
    seen.add(parentName);
    if (interfaceCallableContract(parent, types, seen) === "allows") return "allows";
  }
  return "unknown";
}

module.exports = {
  callableReturnContract,
};
