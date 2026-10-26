"use strict";

const { unwrapTransparentParent } = require("./async-ast");
const { createConsumerProof } = require("./async-consumer-proof");
const { namedCallbackDisposition } = require("./async-disposition-callbacks");
const { propertiesWritten, returnsThroughMock } = require("./async-disposition-mocks");
const { compileTargets, targetMatches } = require("./async-patterns");
const { createTargetMatcher } = require("./async-targets");

function createAwaitingConsumers(context, mappedArgumentDisposed, sourceFacts, promiseSlotHandoff) {
  const matcher = createTargetMatcher(
    context,
    "awaitingConsumers",
    (id) => !propertiesWritten(id, context, new Set(), null, true),
    true,
  );
  const consumers = (context.options?.[0]?.awaitingConsumers || []).map((consumer) => ({
    targets: compileTargets({ targets: [consumer] }, "targets"),
    indexes: consumer.callbackArgumentIndexes || [],
    properties: consumer.callbackArgumentProperties || [],
  }));
  const namedResults = new WeakMap();
  const provenAwaitingConsumer = createConsumerProof(context, sourceFacts);

  function configuredConsumer(fn) {
    const callback = unwrapTransparentParent(fn);
    const target = matcher.resolveCallTarget(callback.parent);
    if (!target) return false;
    const index = target.arguments.indexOf(callback);
    if (index < 0 || target.arguments.slice(0, index).some((arg) => arg.type === "SpreadElement")) {
      return false;
    }
    return consumers.some(
      (consumer) =>
        consumer.indexes.includes(index) &&
        targetMatches(consumer.targets, target.source, target.calleeName),
    );
  }

  function returnsThroughConsumer(fn) {
    return configuredConsumer(fn) || provenAwaitingConsumer(fn);
  }

  function propertyThroughConsumer(argument) {
    const value = unwrapTransparentParent(argument);
    const property = value.parent;
    if (
      property?.type !== "Property" ||
      property.value !== value ||
      property.computed ||
      property.method ||
      property.kind !== "init"
    )
      return false;
    const object = property.parent;
    if (
      object?.type !== "ObjectExpression" ||
      object.properties.some(
        (entry) =>
          entry.type !== "Property" || entry.computed || entry.method || entry.kind !== "init",
      )
    )
      return false;
    const names = object.properties.map((entry) => entry.key.name ?? entry.key.value);
    if (new Set(names).size !== names.length) return false;
    const callback = unwrapTransparentParent(object);
    const target = matcher.resolveCallTarget(callback.parent);
    if (!target || target.arguments.some((arg) => arg.type === "SpreadElement")) return false;
    const index = target.arguments.indexOf(callback);
    const name = property.key.name ?? property.key.value;
    return consumers.some(
      (consumer) =>
        targetMatches(consumer.targets, target.source, target.calleeName) &&
        consumer.properties.some(
          (slot) => slot.argumentIndex === index && slot.propertyNames.includes(name),
        ),
    );
  }

  function namedDisposition(fn) {
    if (!namedResults.has(fn)) {
      // Aggregate proofs can re-enter this function through a returned mapper.
      namedResults.set(fn, false);
      namedResults.set(
        fn,
        namedCallbackDisposition(
          fn,
          context,
          (argument) =>
            returnsThroughMock(argument, context) ||
            configuredConsumer(argument) ||
            propertyThroughConsumer(argument) ||
            provenAwaitingConsumer(argument, context) ||
            promiseSlotHandoff(argument) ||
            mappedArgumentDisposed(fn, argument),
        ),
      );
    }
    return namedResults.get(fn);
  }

  return { visitors: matcher.visitors, returnsThroughConsumer, namedDisposition };
}

module.exports = { createAwaitingConsumers };
