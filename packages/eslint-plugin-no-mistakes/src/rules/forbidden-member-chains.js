"use strict";

const { ruleSuppression } = require("../rule-suppression");
const { schema, compile, assertChains } = require("./forbidden-member-chains-schema");
const {
  constSegment,
  computeChain,
  disposition,
  hasMatchingAncestor,
  invocationOf,
} = require("./forbidden-member-chains-match");
const { destructureVisitors } = require("./forbidden-member-chains-destructure");

module.exports = {
  meta: {
    type: "problem",
    docs: { description: "ban configured static member-access chains", recommended: false },
    schema,
    messages: {
      forbiddenChain: "{{message}}",
      unanalyzable: "Cannot tell whether this member chain is `{{path}}`. {{message}}",
    },
  },
  create(context) {
    const compiled = [];
    const entries = context.options[0]?.chains;
    assertChains(entries);
    if (Array.isArray(entries)) {
      for (const entry of entries) compiled.push(compile(entry));
    }
    if (compiled.length === 0) return {};
    const suppressed = ruleSuppression(context, "forbidden-member-chains");
    const resolveName = (node) => constSegment(node, context);
    const cache = new WeakMap();
    const info = (member) => {
      if (cache.has(member)) return cache.get(member);
      const value = computeChain(member, resolveName);
      cache.set(member, value);
      return value;
    };
    const report = (node, chain, kind) => {
      if (suppressed(node)) return;
      if (kind === "match") {
        context.report({ node, messageId: "forbiddenChain", data: { message: chain.message } });
        return;
      }
      context.report({
        node,
        messageId: "unanalyzable",
        data: { path: chain.path, message: chain.message },
      });
    };
    return {
      MemberExpression(node) {
        for (const chain of compiled) {
          const kind = disposition(info(node), chain);
          if (!kind || hasMatchingAncestor(node, chain, info, kind)) continue;
          report(invocationOf(node) || node, chain, kind);
        }
      },
      ...destructureVisitors({ compiled, context, suppressed, info, resolveName }),
    };
  },
};
