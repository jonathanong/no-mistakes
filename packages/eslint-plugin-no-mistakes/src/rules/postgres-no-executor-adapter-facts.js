"use strict";

const contexts = new WeakMap();

function targetFactsFor(context) {
  let facts = contexts.get(context);
  if (!facts) {
    facts = new WeakMap();
    facts.nonWritingCallees = context.options?.[0]?.nonWritingCallees ?? [];
    contexts.set(context, facts);
  }
  return facts;
}

module.exports = { targetFactsFor };
