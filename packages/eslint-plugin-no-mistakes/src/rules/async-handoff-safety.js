"use strict";
const { resolveVariable, memberPropertyName } = require("./async-target-bindings");
const { sourceSlot } = require("./async-source-slots");
const { annotation } = require("./async-source-types");
const { propertiesWritten } = require("./async-disposition-mocks");
const { localBodySafe } = require("./async-consumer-local-proof");

// A typeof annotation carries responsibility only while visible uses preserve
// it. An opaque forwarding call cannot erase the original callback finding.
function createHandoffSafety(proof) {
  const results = new WeakMap();
  let depth = 0;
  function safeSlot(slot, target) {
    if (depth >= 32) return false;
    const { consumer, binding, property } = slot;
    if (!localBodySafe(consumer.node, consumer.context)) return false;
    if (!results.has(binding)) results.set(binding, new Map());
    const cache = results.get(binding);
    if (cache.has(property)) return cache.get(property);
    cache.set(property, false);
    depth++;
    const safe = uses(binding, property, consumer.context, target, new Set());
    depth--;
    cache.set(property, safe);
    return safe;
  }
  function uses(binding, property, owner, target, seen) {
    if (!binding || seen.has(binding) || seen.size >= 32) return false;
    seen = new Set(seen).add(binding);
    const reads = binding.references.filter((ref) => !ref.init);
    return (
      reads.length > 0 &&
      reads.every((ref) => {
        if (ref.isWrite()) return false;
        let value = ref.identifier;
        const member = value.parent;
        if (property !== null && member?.type === "MemberExpression" && member.object === value) {
          const name = memberPropertyName(member);
          if (name === null) return false;
          if (name !== property) return true;
          return flow(member, null, owner, target, seen);
        }
        return flow(value, property, owner, target, seen);
      })
    );
  }
  function returned(value, property, owner, target) {
    let fn = value.parent;
    while (
      fn &&
      !["FunctionDeclaration", "FunctionExpression", "ArrowFunctionExpression"].includes(fn.type)
    )
      fn = fn.parent;
    const type = fn?.returnType?.typeAnnotation;
    if (!type) return true;
    const selected =
      property === null ? { type, context: owner } : proof.property(type, owner, property);
    return selected && proof.targetType(selected.type, selected.context) === target;
  }
  function flow(value, property, owner, target, seen) {
    const parent = value.parent;
    if (!parent) return false;
    if (parent.type.startsWith("TS") && parent.expression === value) {
      return (
        property === null &&
        proof.targetType(parent.typeAnnotation, owner) === target &&
        flow(parent, property, owner, target, seen)
      );
    }
    if (parent.type === "LogicalExpression" && parent.operator === "??" && parent.left === value) {
      return (
        property === null &&
        proof.target(parent.right, owner) === target &&
        flow(parent, property, owner, target, seen)
      );
    }
    if (parent.type === "CallExpression") {
      if (parent.callee === value) return property === null && !parent.optional;
      const next = sourceSlot(value, owner, proof, property);
      return next && proof.targetType(next.type, next.context) === target && safeSlot(next, target);
    }
    if (
      parent.type === "VariableDeclarator" &&
      parent.init === value &&
      parent.id.type === "Identifier" &&
      parent.parent.kind === "const"
    ) {
      if (propertiesWritten(parent.id, owner, new Set(), null, true)) return false;
      const type = annotation(parent.id);
      if (type) {
        const selected =
          property === null ? { type, context: owner } : proof.property(type, owner, property);
        if (!selected || proof.targetType(selected.type, selected.context) !== target) return false;
      }
      return uses(resolveVariable(parent.id, owner), property, owner, target, seen);
    }
    if (parent.type === "ReturnStatement" && parent.argument === value)
      return returned(value, property, owner, target);
    if (parent.type === "ArrowFunctionExpression" && parent.body === value)
      return returned(value, property, owner, target);
    if (
      parent.type === "Property" &&
      parent.value === value &&
      property === null &&
      !parent.computed &&
      !parent.method &&
      parent.kind === "init"
    ) {
      const object = parent.parent;
      const names = object.properties.map((item) =>
        item.type === "Property" && !item.computed ? (item.key.name ?? item.key.value) : null,
      );
      if (names.includes(null) || new Set(names).size !== names.length) return false;
      return flow(object, parent.key.name ?? parent.key.value, owner, target, seen);
    }
    if (parent.type === "SpreadElement" && parent.argument === value && property !== null) {
      const object = parent.parent;
      if (
        object.type !== "ObjectExpression" ||
        object.properties.some(
          (item) =>
            item !== parent &&
            (item.type !== "Property" ||
              item.computed ||
              (item.key.name ?? item.key.value) === property),
        )
      )
        return false;
      return flow(object, property, owner, target, seen);
    }
    return false;
  }
  return safeSlot;
}

module.exports = { createHandoffSafety };
