"use strict";

const { unwrapTransparentParent } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { createImportedSources } = require("./async-consumer-imported-proof");
const { annotation, createSourceTypes } = require("./async-source-types");
const { propertiesWritten } = require("./async-disposition-mocks");
const { sourceSlot } = require("./async-source-slots");
const { createHandoffSafety } = require("./async-handoff-safety");

// This source projection shares imported source parsing with the body proof. It
// never constructs a TypeScript program or depends on the runner's argv.
function createPromiseHandoff(context, sourceFacts) {
  let types;
  let safeSlot;
  function sourceTypes() {
    if (context.sourceCode.parserServices?.program?.getCompilerOptions().noResolve) return null;
    return (types ||= createSourceTypes(
      context,
      (sourceFacts.sources ||= createImportedSources(context)),
    ));
  }
  function promiseSlotHandoff(node) {
    const proof = sourceTypes();
    if (!proof) return false;
    const receiving = sourceSlot(node, context, proof);
    const target = receiving && proof.targetType(receiving.type, receiving.context);
    if (!target || !(safeSlot ||= createHandoffSafety(proof))(receiving, target)) return false;
    const variable = resolveVariable(unwrapTransparentParent(node), context);
    const declared = variable?.defs.length === 1 && annotation(variable.defs[0].name);
    // An explicit callback annotation must preserve the receiving target identity.
    return !declared || proof.targetType(declared, context) === target;
  }
  function promiseTypedTargetCall(node) {
    const proof = sourceTypes();
    if (!proof) return false;
    const type = proof.valueType(node.callee, context);
    if (!type || !proof.targetType(type.type, type.context)) return false;
    let root = node.callee;
    while (root.type === "MemberExpression") root = root.object;
    return !propertiesWritten(root, context, new Set(), null, true);
  }
  return { promiseSlotHandoff, promiseTypedTargetCall };
}

module.exports = { createPromiseHandoff };
