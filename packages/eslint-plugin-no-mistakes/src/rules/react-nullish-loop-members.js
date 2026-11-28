"use strict";

const { unwrapSequence } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { assertedObjectFacts } = require("./react-nullish-props");

function contains(root, node) {
  for (let current = node; current; current = current.parent) if (current === root) return true;
  return false;
}

function parameterReady(def, use) {
  const params = def.node.params ?? [];
  const sourceIndex = params.findIndex((param) => contains(param, def.name));
  const useIndex = params.findIndex((param) => contains(param, use));
  if (useIndex < 0) return def.node.body.range[0] < use.range[0];
  return sourceIndex >= 0 && sourceIndex < useIndex;
}

function notePair(primary, aliases, binding, value) {
  if (!value) return;
  primary.set(binding, value);
  aliases.set(binding, value);
}

function objectAliasProps(node, facts, context, objectBindings, unresolvedBindings) {
  const asserted = assertedObjectFacts(node.init, facts);
  if (asserted) return node.parent.kind === "const" ? asserted : null;
  const init = unwrapSequence(node.init);
  if (node.parent.kind !== "const" || init?.type !== "Identifier") return null;
  const source = resolveVariable(init, context);
  const target = resolveVariable(node.id, context);
  const reassigned = (binding) => binding.references.some((ref) => ref.isWrite() && !ref.init);
  if (!source || source.defs.length !== 1 || reassigned(source) || reassigned(target)) return null;
  const def = source.defs[0];
  // Sequential name facts must not cross a TDZ shadow or a parameter initializer.
  const initialized =
    def.type === "Parameter"
      ? parameterReady(def, init)
      : def.type === "Variable" && def.parent.kind === "const" && def.node.range[1] < init.range[0];
  return initialized
    ? { props: objectBindings.get(source), unresolved: unresolvedBindings.get(source) }
    : null;
}

module.exports = { notePair, objectAliasProps };
