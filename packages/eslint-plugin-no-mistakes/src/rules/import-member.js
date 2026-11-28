"use strict";

const { unwrapExpression } = require("./async-ast");
const { memberPropertyName } = require("./logical-call");

function namedFact(moduleName, names) {
  return {
    name: names.join("."),
    module: moduleName,
    kind: "named",
    imported: names.at(-1),
    typeOnly: false,
  };
}

function memberFact(node, context, importFact, requireModule, namespaceMember, seen, memo) {
  const names = [];
  let dynamic = false;
  let current = unwrapExpression(node);
  while (current?.type === "MemberExpression") {
    const property = memberPropertyName(current);
    if (property) names.push(property);
    else dynamic = true;
    current = unwrapExpression(current.object);
  }
  if (!names.length && !dynamic) return null;
  names.reverse();
  const moduleName = requireModule(current, context);
  if (typeof moduleName === "string") {
    return dynamic ? { unresolved: true } : namedFact(moduleName, names);
  }
  if (moduleName === null) return { unresolved: true };
  const base = importFact(current, context, seen, memo);
  if (!base || base.typeOnly) return null;
  if (dynamic || base.unresolved) return { unresolved: true };
  let fact = base;
  for (const name of names) {
    const next = namespaceMember(fact, name);
    if (next?.unresolved) return { unresolved: true };
    if (!next) {
      if (fact.kind !== "named" || !(fact.name || fact.imported)) return null;
      fact = { ...fact, name: `${fact.name || fact.imported}.${name}`, imported: name };
      continue;
    }
    fact = fact.name && !next.name ? { ...next, name: `${fact.name}.${name}` } : next;
  }
  return { ...fact, name: fact.name || names.join(".") };
}

function rootIdentifier(node) {
  let current = unwrapExpression(node);
  while (current?.type === "MemberExpression") current = unwrapExpression(current.object);
  return current?.type === "Identifier" ? current : null;
}

module.exports = { memberFact, rootIdentifier };
