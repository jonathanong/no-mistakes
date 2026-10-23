"use strict";
const { ruleSuppression } = require("../rule-suppression");
const { logicalCall } = require("./module-mock-helpers");
const { hasBannedName } = require("./no-banned-import-outside-allowed-paths-config");
const { tagForExpression } = require("./no-banned-import-outside-allowed-paths-tags");

function createInvocationChecker(context, config, activeAliases) {
  const suppressed = ruleSuppression(context, "no-banned-import-outside-allowed-paths");
  function reportCall(node, module, name) {
    if (!suppressed(node))
      context.report({ node, messageId: "bannedImport", data: { module, name } });
  }

  // Shared by CallExpression and NewExpression: a banned capability is
  // just as reachable through `new BannedClient()` as through
  // `BannedClient()`, and both invocation forms resolve their callee the
  // same way.
  function checkInvocation(node) {
    const aliases = activeAliases();
    const direct = tagForExpression(node.callee, context, aliases, config);
    const invoked = !direct && logicalCall(node);
    const tag = direct || (invoked && tagForExpression(invoked.callee, context, aliases, config));
    if (tag?.kind === "direct") {
      reportCall(node.callee, tag.module, tag.name);
      return;
    }
    if (tag?.kind !== "object") return;
    for (const module of tag.modules) {
      if (hasBannedName(config, module, "default")) {
        reportCall(node.callee, module, "default");
        return;
      }
    }
  }

  return checkInvocation;
}
module.exports = { createInvocationChecker };
