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

function memberFact(node, context, importFact, requireModule, namespaceMember) {
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
  const base = importFact(current, context);
  if (!base) return null;
  if (dynamic || base.unresolved) return { unresolved: true };
  let fact = base;
  let projected = false;
  for (const name of names) {
    const next = namespaceMember(fact, name);
    if (next?.unresolved || (projected && !next)) return { unresolved: true };
    if (!next) return null;
    fact = next;
    projected = true;
  }
  return { ...fact, name: fact.name || names.join(".") };
}

module.exports = { memberFact };
