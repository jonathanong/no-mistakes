"use strict";

const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { createImportedSources } = require("./async-consumer-imported-proof");
const { annotation, createSourceTypes } = require("./async-source-types");
const { sourceSlot } = require("./async-source-slots");
const { createHandoffSafety } = require("./async-handoff-safety");
const { createReceiverProof } = require("./async-consumer-receivers");
const { localBodySafe } = require("./async-consumer-local-proof");
const { targetReturns } = require("./async-handoff-return");

// This source projection shares imported source parsing with the body proof. It
// never constructs a TypeScript program or depends on the runner's argv.
function createPromiseHandoff(context, sourceFacts) {
  let types;
  let safeSlot;
  function sourceTypes() {
    if (context.sourceCode.parserServices?.program?.getCompilerOptions().noResolve) return null;
    // Share the body's unavailable-tooling result as well as its successful source session.
    if (sourceFacts.sources === null) sourceFacts.sources = createImportedSources(context) || false;
    if (!sourceFacts.sources) return null;
    return (types ||= createSourceTypes(
      sourceFacts.targets,
      sourceFacts.sources,
      (sourceFacts.receiverShape ||= createReceiverProof(sourceFacts)),
    ));
  }
  function promiseSlotHandoff(node) {
    const proof = sourceTypes();
    if (!proof) return false;
    const callback = proof.callable(unwrapExpression(node), context);
    if (!callback || !localBodySafe(callback.node, callback.context)) return false;
    function receiverFree(value) {
      if (!value?.type) return true;
      if (value.type === "ThisExpression") return false;
      return (callback.context.sourceCode.visitorKeys[value.type] || []).every((key) =>
        Array.isArray(value[key]) ? value[key].every(receiverFree) : receiverFree(value[key]),
      );
    }
    if (!receiverFree(callback.node) || !targetReturns(callback.node, callback.context, proof))
      return false;
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
    return Boolean(type && proof.targetType(type.type, type.context));
  }
  return { promiseSlotHandoff, promiseTypedTargetCall };
}

module.exports = { createPromiseHandoff };
