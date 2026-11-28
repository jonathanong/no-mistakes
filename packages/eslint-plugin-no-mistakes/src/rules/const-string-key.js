"use strict";

const { unwrapSequence } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { literalString } = require("./logical-call");

function constStringKey(node, context) {
  const current = unwrapSequence(node);
  if (!context?.sourceCode?.getScope || current?.type !== "Identifier") return null;
  const variable = resolveVariable(current, context);
  const def = variable?.defs?.length === 1 ? variable.defs[0] : null;
  if (def?.type !== "Variable" || def.parent?.kind !== "const") return null;
  const declarator = def.node;
  if (declarator?.id?.type !== "Identifier" || !declarator.init) return null;
  return literalString(declarator.init);
}

module.exports = { constStringKey };
