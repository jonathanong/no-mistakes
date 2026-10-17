"use strict";

const { ruleSuppression } = require("../rule-suppression");
const { schema, compile } = require("./forbidden-member-chains-schema");
const {
  matches,
  computeChain,
  hasMatchingAncestor,
  invocationOf,
} = require("./forbidden-member-chains-match");
const { destructureVisitors } = require("./forbidden-member-chains-destructure");

module.exports = {
  meta: {
    type: "problem",
    docs: { description: "ban configured static member-access chains", recommended: false },
    schema,
    messages: { forbiddenChain: "{{message}}" },
  },
  create(context) {
    const compiled = [];
    const entries = context.options[0]?.chains;
    if (Array.isArray(entries)) {
      for (const entry of entries) compiled.push(compile(entry));
    }
    if (compiled.length === 0) return {};
    const suppressed = ruleSuppression(context, "forbidden-member-chains");
    const cache = new WeakMap();
    const info = (member) => {
      if (cache.has(member)) return cache.get(member);
      const value = computeChain(member);
      cache.set(member, value);
      return value;
    };
    const report = (node, chain) => {
      if (suppressed(node)) return;
      context.report({ node, messageId: "forbiddenChain", data: { message: chain.message } });
    };
    return {
      MemberExpression(node) {
        for (const chain of compiled) {
          if (!matches(info(node), chain) || hasMatchingAncestor(node, chain, info)) continue;
          report(invocationOf(node) || node, chain);
        }
      },
      ...destructureVisitors({ compiled, context, suppressed, info }),
    };
  },
};
