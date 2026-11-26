"use strict";

const UNANALYZABLE_MESSAGE =
  "This PostgreSQL value is not a static import, require, or import-equals binding. Write the module specifier as a string literal so the rule can judge it. A dynamic module is not a clean result.";

const unanalyzableSchema = { enum: ["advise", "report", "ignore"] };

function unanalyzablePolicy(options) {
  const policy = options?.unanalyzable;
  return policy === "report" || policy === "ignore" ? policy : "advise";
}

function reportUnanalyzable(context, node, policy) {
  if (policy === "ignore" || !node) return;
  context.report({ node, messageId: "unanalyzable" });
}

module.exports = {
  UNANALYZABLE_MESSAGE,
  reportUnanalyzable,
  unanalyzablePolicy,
  unanalyzableSchema,
};
