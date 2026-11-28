"use strict";

const { literalString } = require("./rules/module-mock-helpers");
const { keyName, reactTypeStatus, typeAnnotation, typeName } = require("./react-node-type-query");

function setChanged(previous, next) {
  if (!previous || previous.size !== next.size) return true;
  for (const value of next) if (!previous.has(value)) return true;
  return false;
}

function absorb(target, source, skip) {
  if (!source) return;
  for (const value of source) if (!skip?.has(value)) target.add(value);
}

function heritageName(heritage) {
  const expression = heritage && heritage.expression;
  return expression && expression.type === "Identifier" ? expression.name : null;
}

function collectMembers(members, facts) {
  const props = new Set();
  const unresolved = new Set();
  const declared = new Set();
  for (const member of members || []) {
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

function noteStatus(status, key, incoming) {
  const previous = status.get(key);
  status.set(key, previous && previous !== incoming ? "unresolved" : incoming);
}

function publish(map, name, next, changed) {
  if (!setChanged(map.get(name), next)) return changed;
  map.set(name, next);
  return true;
}

function assignReactObjectFacts(
  program,
  topLevelDeclaration,
  facts,
  objectProps,
  unresolvedObjectProps,
) {
  let objectsChanged = true;
  while (objectsChanged) {
    objectsChanged = false;
    const grouped = new Map();
    const aliases = new Map();
    for (const statement of program.body || []) {
      const declaration = topLevelDeclaration(statement);
      const literal = declaration.typeAnnotation?.type === "TSTypeLiteral";
      const objectLike =
        declaration.type === "TSInterfaceDeclaration" ||
        (declaration.type === "TSTypeAliasDeclaration" && literal);
      if (objectLike) {
        const group = grouped.get(declaration.id.name) ?? [];
        group.push(declaration);
        grouped.set(declaration.id.name, group);
      }
      const target =
        declaration.type === "TSTypeAliasDeclaration" ? typeName(declaration.typeAnnotation) : null;
      if (!target || target.includes(".") || target === declaration.id.name) continue;
      const list = aliases.get(declaration.id.name) ?? [];
      list.push(target);
      aliases.set(declaration.id.name, list);
    }
    const names = new Set([...grouped.keys(), ...aliases.keys()]);
    for (const name of names) {
      const status = new Map();
      const inheritedProps = new Set();
      const inheritedUnresolved = new Set();
      for (const declaration of grouped.get(name) ?? []) {
        const collected = collectMembers(
          declaration.type === "TSTypeAliasDeclaration"
            ? declaration.typeAnnotation.members
            : declaration.body?.body,
          facts,
        );
        for (const key of collected.declared) {
          const incoming = collected.props.has(key)
            ? "yes"
            : collected.unresolved.has(key)
              ? "unresolved"
              : "no";
          noteStatus(status, key, incoming);
        }
        for (const heritage of declaration.extends || []) {
          const inheritedName = heritageName(heritage);
          absorb(inheritedProps, objectProps.get(inheritedName));
          absorb(inheritedUnresolved, unresolvedObjectProps.get(inheritedName));
        }
      }
      let sawAlias = false;
      for (const target of aliases.get(name) ?? []) {
        if (!objectProps.has(target) && !unresolvedObjectProps.has(target)) continue;
        sawAlias = true;
        for (const key of objectProps.get(target) ?? []) noteStatus(status, key, "yes");
        for (const key of unresolvedObjectProps.get(target) ?? [])
          noteStatus(status, key, "unresolved");
      }
      if (!grouped.has(name) && !sawAlias) continue;
      const props = new Set();
      const unresolved = new Set();
      for (const [key, value] of status) {
        if (value === "yes") props.add(key);
        else if (value === "unresolved") unresolved.add(key);
      }
      for (const key of inheritedProps) if (!status.has(key)) props.add(key);
      for (const key of inheritedUnresolved)
        if (!status.has(key) && !props.has(key)) unresolved.add(key);
      objectsChanged = publish(objectProps, name, props, objectsChanged);
      objectsChanged = publish(unresolvedObjectProps, name, unresolved, objectsChanged);
    }
  }
}

module.exports = { assignReactObjectFacts };
