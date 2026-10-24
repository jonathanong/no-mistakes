"use strict";

const { unwrapTransparentParent } = require("./async-ast");
const { namedCallbackDisposition } = require("./async-disposition-callbacks");
const { propertiesWritten, returnsThroughMock } = require("./async-disposition-mocks");
const { compileTargets, targetMatches } = require("./async-patterns");
const { createTargetMatcher } = require("./async-targets");

function createAwaitingConsumers(context) {
  const matcher = createTargetMatcher(
    context,
    "awaitingConsumers",
    (id) => !propertiesWritten(id, context, new Set(), null, true),
    true,
  );
  const consumers = (context.options?.[0]?.awaitingConsumers || []).map((consumer) => ({
    targets: compileTargets({ targets: [consumer] }, "targets"),
    indexes: consumer.callbackArgumentIndexes,
  }));
  const namedResults = new WeakMap();

  function returnsThroughConsumer(fn) {
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

  function namedDisposition(fn) {
    if (!namedResults.has(fn)) {
      namedResults.set(
        fn,
        namedCallbackDisposition(
          fn,
          context,
          (argument) => returnsThroughMock(argument, context) || returnsThroughConsumer(argument),
        ),
      );
    }
    return namedResults.get(fn);
  }

  return { visitors: matcher.visitors, returnsThroughConsumer, namedDisposition };
}

module.exports = { createAwaitingConsumers };
