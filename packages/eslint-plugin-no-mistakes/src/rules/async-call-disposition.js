"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const {
  collectedPromise,
  isPromiseAllCall,
  retainedPromise,
} = require("./async-disposition-bindings");
const {
  directlyDisposed,
  mappedArgumentDisposed,
  observedExpression,
  promiseAllDisposed,
  mappedPromise,
} = require("./async-disposition-expressions");
const { dispositionOptionsSchema } = require("./async-schema");
const { createTargetMatcher } = require("./async-targets");
const { createAwaitingConsumers } = require("./async-disposition-consumers");

function isDisposed(node, context, consumers, seen = new Set()) {
  if (seen.has(node)) return false;
  seen.add(node);
  const observed = observedExpression(node, context);
  const disposed = (value) => isDisposed(value, context, consumers, seen);
  return (
    directlyDisposed(node, context, consumers) ||
    directlyDisposed(observed, context, consumers) ||
    promiseAllDisposed(observed, context, consumers) ||
    retainedPromise(observed, context, disposed) ||
    mappedPromise(observed, context, consumers, disposed) ||
    collectedPromise(
      observed,
      context,
      (value) => isPromiseAllCall(value, context),
      disposed,
      node.type === "CallExpression",
    )
  );
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "require explicit async disposition for configured async calls",
      recommended: false,
    },
    hasSuggestions: true,
    schema: dispositionOptionsSchema,
    messages: {
      disposition:
        "Handle this async promise explicitly with await, return, Promise.all(...), or void.",
      markVoid: "Prefix the call with void to mark this promise as intentionally floating.",
    },
  },
  (context) => {
    const matcher = createTargetMatcher(context);
    const consumers = createAwaitingConsumers(context, (fn, argument) =>
      mappedArgumentDisposed(fn, argument, context, consumers, (value) =>
        isDisposed(value, context, consumers),
      ),
    );
    const suppressed = ruleSuppression(context, "async-call-disposition");
    if (!matcher.hasTargets) return {};

    return {
      ...Object.fromEntries(
        Object.keys(matcher.visitors).map((key) => [
          key,
          (node) => {
            matcher.visitors[key](node);
            consumers.visitors[key](node);
          },
        ]),
      ),
      CallExpression(node) {
        if (!matcher.isTargetCall(node) || suppressed(node) || isDisposed(node, context, consumers))
          return;
        context.report({
          node,
          messageId: "disposition",
          suggest:
            node.parent?.type === "ExpressionStatement"
              ? [
                  {
                    messageId: "markVoid",
                    fix(fixer) {
                      return fixer.insertTextBefore(node, "void ");
                    },
                  },
                ]
              : [],
        });
      },
    };
  },
);
