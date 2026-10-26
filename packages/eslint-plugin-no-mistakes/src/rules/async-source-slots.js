"use strict";
const { unwrapTransparentParent } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { annotation } = require("./async-source-types");
const { propertiesWritten } = require("./async-disposition-mocks");

function sourceSlot(argument, context, proof, selected = null) {
  let value = unwrapTransparentParent(argument);
  let property = selected;
  if (value.parent?.type === "Property") {
    const entry = value.parent;
    const object = entry.parent;
    if (
      property !== null ||
      entry.value !== value ||
      object.type !== "ObjectExpression" ||
      object.properties.some(
        (item) => item.type !== "Property" || item.computed || item.method || item.kind !== "init",
      )
    )
      return null;
    const names = object.properties.map((item) => item.key.name ?? item.key.value);
    if (new Set(names).size !== names.length) return null;
    property = entry.key.name ?? entry.key.value;
    value = unwrapTransparentParent(object);
  }
  const call = value.parent;
  if (
    call?.type !== "CallExpression" ||
    call.optional ||
    call.arguments.some((arg) => arg.type === "SpreadElement")
  )
    return null;
  const index = call.arguments.indexOf(value);
  const consumer = proof.callable(call.callee, context);
  if (!consumer || index < 0) return null;
  const parameter = consumer.node.params[index];
  const param = parameter?.type === "AssignmentPattern" ? parameter.left : parameter;
  const binding = param?.type === "Identifier" && resolveVariable(param, consumer.context);
  if (
    !binding ||
    consumer.node.params.some((item) => item.type === "RestElement") ||
    binding.references.some((ref) => ref.isWrite() && !ref.init) ||
    propertiesWritten(param, consumer.context, new Set(), null, true)
  )
    return null;
  const type = annotation(param);
  const receiving =
    property === null
      ? { type, context: consumer.context }
      : proof.property(type, consumer.context, property);
  return receiving && { ...receiving, consumer, param, property, binding };
}

module.exports = { sourceSlot };
