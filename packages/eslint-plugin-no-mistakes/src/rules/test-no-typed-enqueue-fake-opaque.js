"use strict";

const { unwrapExpression } = require("./async-ast");
const { isReassigned } = require("./async-target-bindings");
const { requireResolution } = require("./async-target-sources");
const { memberPropertyName } = require("./logical-call");
const {
  frameworkModules,
  patternExportName,
  referenceBinding,
  scopeContext,
} = require("./test-no-typed-enqueue-fake-helpers");

function unresolvedRequire(node, context) {
  node = unwrapExpression(node);
  const resolved = requireResolution(node, context);
  if (resolved) return resolved.source === null;
  if (node?.type !== "Identifier") return false;
  const binding = referenceBinding(node, context.sourceCode);
  const variable = binding?.defs.length === 1 ? binding.defs[0] : null;
  if (
    variable?.type !== "Variable" ||
    variable.node?.id?.type !== "Identifier" ||
    isReassigned(node, context)
  ) {
    return false;
  }
  return requireResolution(variable.node.init, context)?.source === null;
}

function opaqueFrameworkReceiver(receiver, sourceCode) {
  receiver = unwrapExpression(receiver);
  const context = scopeContext(sourceCode);
  if (receiver?.type === "MemberExpression") {
    return (
      frameworkModules.has(memberPropertyName(receiver)) &&
      unresolvedRequire(unwrapExpression(receiver.object), context)
    );
  }
  if (receiver?.type !== "Identifier") return false;
  const binding = referenceBinding(receiver, sourceCode);
  const variable = binding?.defs.length === 1 ? binding.defs[0] : null;
  if (
    variable?.type !== "Variable" ||
    variable.node?.id?.type !== "ObjectPattern" ||
    isReassigned(receiver, context)
  ) {
    return false;
  }
  return (
    frameworkModules.has(patternExportName(variable.node.id, receiver.name)) &&
    requireResolution(variable.node.init, context)?.source === null
  );
}

module.exports = { opaqueFrameworkReceiver };
