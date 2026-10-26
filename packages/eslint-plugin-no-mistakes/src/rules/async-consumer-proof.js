"use strict";

const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { importedAwaited } = require("./async-consumer-imported-proof");
const { localFunction, bodyAwaited } = require("./async-consumer-local-proof");
const { adoptReturned } = require("./async-consumer-property-carrier");
const {
  forwarded,
  propertyBodyAwaited,
  propertyWritten,
} = require("./async-consumer-property-proof");
const { createReceiverProof } = require("./async-consumer-receivers");

function createConsumerProof(context) {
  const results = new WeakMap();
  const sourceFacts = { typed: new WeakMap(), sources: null };
  const receiverShape = createReceiverProof(sourceFacts);
  const noParameter = Object.create(null);
  let depth = 0;
  return function provenAwaitingConsumer(argument) {
    let value = unwrapTransparentParent(argument);
    let property = null;
    if (value.parent?.type === "Property") {
      const entry = value.parent;
      const object = entry.parent;
      if (
        entry.value !== value ||
        object.type !== "ObjectExpression" ||
        object.properties.some(
          (item) =>
            item.type !== "Property" || item.computed || item.method || item.kind !== "init",
        )
      )
        return false;
      const names = object.properties.map((item) => item.key.name ?? item.key.value);
      if (new Set(names).size !== names.length) return false;
      property = entry.key.name ?? entry.key.value;
      value = unwrapTransparentParent(object);
    }
    const call = value.parent;
    const mode = proveCall(call, call?.arguments?.indexOf(value), property, context);
    // A returned dependencies object still has to be awaited by this caller.
    if (mode?.returned) {
      return (
        adoptReturned(call, mode, property, context, {
          proveCall,
          receiverShape,
          propertyWritten,
          forward: forwarded,
          variable: noParameter,
        }) === true
      );
    }
    return mode === true;
  };

  function withAsync(fn, mode) {
    return mode?.returned ? { ...mode, async: Boolean(fn.async) } : mode;
  }

  function proveBody(fn, index, owner, property, shape) {
    return property === null
      ? bodyAwaited(fn, index, owner)
      : propertyBodyAwaited(fn, index, owner, property, proveCall, receiverShape, shape);
  }
  function proveCall(call, index, property, owner, shape) {
    if (
      call?.type !== "CallExpression" ||
      call.optional ||
      call.callee.type !== "Identifier" ||
      call.arguments.some((arg) => arg.type === "SpreadElement") ||
      index < 0 ||
      depth >= 32
    )
      return false;
    if (!shape) {
      const argument = unwrapExpression(call.arguments[index]);
      shape = argument.type === "ObjectExpression" ? receiverShape(argument, owner) : new Map();
    }
    // A custom prototype can supply an otherwise absent optional method.
    if (shape.has("__proto__")) return false;
    const binding = resolveVariable(call.callee, owner);
    if (!binding) return false;
    if (!results.has(binding)) results.set(binding, new Map());
    const slots = results.get(binding);
    const key = JSON.stringify([index, property, [...shape].sort()]);
    if (!slots.has(key)) {
      // Mark before recursion: cyclic forwarding cannot establish a contract.
      slots.set(key, false);
      depth++;
      const localFn = localFunction(unwrapExpression(call.callee), owner);
      let mode = false;
      if (localFn) mode = withAsync(localFn, proveBody(localFn, index, owner, property, shape));
      if (!mode) {
        mode = importedAwaited(
          call,
          index,
          owner,
          sourceFacts,
          property,
          (fn, index, owner, property) => proveBody(fn, index, owner, property, shape),
        );
      }
      slots.set(key, mode || false);
      depth--;
    }
    return slots.get(key);
  }
}

module.exports = { createConsumerProof };
