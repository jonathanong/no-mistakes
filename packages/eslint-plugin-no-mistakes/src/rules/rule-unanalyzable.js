"use strict";

const UNANALYZABLE_SCHEMA = { enum: ["finding", "ignore"] };

function reportsUnanalyzable(context) {
  return (context.options?.[0]?.unanalyzable ?? "finding") === "finding";
}

function reportUnanalyzable(context, node, subject) {
  if (!reportsUnanalyzable(context) || !node) return false;
  context.report({
    node,
    messageId: "unanalyzable",
    data: { subject },
  });
  return true;
}

module.exports = { UNANALYZABLE_SCHEMA, reportUnanalyzable };
