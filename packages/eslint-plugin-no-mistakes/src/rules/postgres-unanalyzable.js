"use strict";

function unanalyzablePolicy(options) {
  const policy = options?.unanalyzable;
  return policy === "report" || policy === "ignore" ? policy : "advise";
}

function reportUnanalyzable(context, node, policy) {
  if (policy === "ignore" || !node) return;
  context.report({ node, messageId: "unanalyzable" });
}

module.exports = {
  reportUnanalyzable,
  unanalyzablePolicy,
};
