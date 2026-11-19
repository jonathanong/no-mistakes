"use strict";

const { aliasHint, directCall, judgedDirect } = require("./postgres-no-executor-adapter-forward");
const { passedAlias, passedDirect } = require("./postgres-no-executor-adapter-passed");

function createReporters(context, suppressed) {
  const source = context.sourceCode;

  function quotedExecutor(quoteNode, name) {
    if (quoteNode?.type !== "Property") return name;
    const key = source.getText(quoteNode.key);
    return `${quoteNode.computed ? `[${key}]` : key}: ${name}`;
  }

  function reportAdapter(node, adapter, call, index, quoteNode) {
    if (suppressed(node)) return;
    const names = adapter.names;
    const leaves = adapter.leaves;
    const rejected = names
      .map((name) => directCall(source, call, index, quotedExecutor(quoteNode, name), quoteNode))
      .join(" or ");
    context.report({
      node,
      messageId: adapter.reason ? "blocked" : "forward",
      data: {
        executor: names.join(" or "),
        direct: judgedDirect(names, leaves, adapter.conditionalText),
        alias: aliasHint(names, leaves),
        rejected,
        reason: adapter.reason ?? "the selector contract changed",
      },
    });
  }

  function reportPassed(node, passed) {
    if (suppressed(node)) return;
    const names = [...new Set(passed.leaves.map((leaf) => leaf.name))];
    context.report({
      node,
      messageId: "passed",
      data: {
        executor: names.join(" or "),
        direct: passedDirect(names, passed.leaves, passed.conditionalText),
        alias: passedAlias(names, passed.leaves),
      },
    });
  }

  return { reportAdapter, reportPassed };
}

module.exports = { createReporters };
