"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const {
  collectedPromise,
  isPromiseAllCall,
  retainedCollection,
  retainedPromise,
} = require("./async-disposition-bindings");
const {
  directlyDisposed,
  observedExpression,
  promiseAllDisposed,
  mappedPromise,
} = require("./async-disposition-expressions");
const { targetOptionsSchema } = require("./async-schema");
const { createTargetMatcher } = require("./async-targets");

function isDisposed(node, context, seen = new Set()) {
  if (seen.has(node)) return false;
  seen.add(node);
  const observed = observedExpression(node, context);
  const disposed = (value) => isDisposed(value, context, seen);
  return (
    directlyDisposed(node, context) ||
    directlyDisposed(observed, context) ||
    promiseAllDisposed(observed, context) ||
    retainedPromise(observed, context, disposed) ||
    mappedPromise(observed, (value) =>
      retainedCollection(
        value,
        context,
        (aggregate) => isPromiseAllCall(aggregate, context),
        disposed,
      ),
    ) ||
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
    schema: targetOptionsSchema,
    messages: {
      disposition:
        "Handle this async promise explicitly with await, return, Promise.all(...), or void.",
      markVoid: "Prefix the call with void to mark this promise as intentionally floating.",
    },
  },
  (context) => {
    const matcher = createTargetMatcher(context);
    const suppressed = ruleSuppression(context, "async-call-disposition");
    if (!matcher.hasTargets) return {};

    return {
      ...matcher.visitors,
      CallExpression(node) {
        if (!matcher.isTargetCall(node) || suppressed(node) || isDisposed(node, context)) return;
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
