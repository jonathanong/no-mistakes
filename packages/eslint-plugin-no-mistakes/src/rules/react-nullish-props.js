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

function classifiedProps(type, facts) {
  if (type && type.type === "TSTypeLiteral") {
    const props = new Set();
    const unresolved = new Set();
    for (const member of type.members) {
      if (member.type !== "TSPropertySignature") continue;
      const name = member.computed ? literalString(member.key) : keyName(member.key);
      if (!name) continue;
      const status = reactTypeStatus(typeAnnotation(member), facts);
      if (status === "yes") props.add(name);
      else if (status === "unresolved") unresolved.add(name);
    }
    return { props, unresolved };
  }
  const name = typeName(type);
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

module.exports = { assertedObjectFacts, classifiedProps, memberExpressionStatus, objectInitFacts };
