"use strict";

const { reactTypeStatus } = require("../react-node-types");

const UNSUBSTITUTED = {};

function appliedStatus(annotation, facts, substitutions) {
  const name =
    annotation?.type === "TSTypeReference" && annotation.typeName?.type === "Identifier"
      ? annotation.typeName.name
      : null;
  let replacement;
  if (name && substitutions)
    for (const [param, value] of substitutions) if (param.name?.name === name) replacement = value;
  if (replacement === undefined) return reactTypeStatus(annotation, facts);
  return replacement === UNSUBSTITUTED ? "unresolved" : reactTypeStatus(replacement, facts);
}

function typeArgumentsOf(type) {
  return type?.typeArguments?.params || type?.typeParameters?.params || [];
}

function withAliasSubstitutions(alias, type, outer) {
  const params = alias.typeParameters?.params ?? [];
  if (!params.length) return outer;
  const args = typeArgumentsOf(type);
  const next = new Map(outer || []);
  for (let index = 0; index < params.length; index += 1)
    next.set(params[index], args[index] ?? params[index].default ?? UNSUBSTITUTED);
  return next;
}

function noteStatus(status, key, incoming) {
  const previous = status.get(key);
  status.set(key, previous && previous !== incoming ? "unresolved" : incoming);
}

function intersectedProps(types, classify) {
  const status = new Map();
  let sawObject = false;
  for (const part of types ?? []) {
    const classified = classify(part);
    if (!classified?.props && !classified?.unresolved) continue;
    sawObject = true;
    const keys =
      classified.declared ??
      new Set([...(classified.props ?? []), ...(classified.unresolved ?? [])]);
    for (const key of keys) {
      const incoming = classified.props?.has(key)
        ? "yes"
        : classified.unresolved?.has(key)
          ? "unresolved"
          : "no";
      noteStatus(status, key, incoming);
    }
  }
  if (!sawObject) return { props: null, unresolved: null };
  const props = new Set();
  const unresolved = new Set();
  for (const [key, value] of status) {
    if (value === "yes") props.add(key);
    else if (value === "unresolved") unresolved.add(key);
  }
  return { props, unresolved, declared: new Set(status.keys()) };
}

module.exports = {
  UNSUBSTITUTED,
  appliedStatus,
  intersectedProps,
  typeArgumentsOf,
  withAliasSubstitutions,
};
