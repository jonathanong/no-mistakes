"use strict";

const { unwrapTransparentParent } = require("./async-ast");
const { compileTargets, targetMatches } = require("./async-patterns");
const { propertiesWritten } = require("./async-disposition-mocks");
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
  return {
    visitors: matcher.visitors,
    returnsThroughConsumer(fn) {
      const callback = unwrapTransparentParent(fn);
      const target = matcher.resolveCallTarget(callback.parent);
      if (!target) return false;
      const index = target.arguments.indexOf(callback);
      if (
        index < 0 ||
        target.arguments.slice(0, index).some((arg) => arg.type === "SpreadElement")
      ) {
        return false;
      }
      return consumers.some(
        (consumer) =>
          consumer.indexes.includes(index) &&
          targetMatches(consumer.targets, target.source, target.calleeName),
      );
    },
  };
}

module.exports = { createAwaitingConsumers };
