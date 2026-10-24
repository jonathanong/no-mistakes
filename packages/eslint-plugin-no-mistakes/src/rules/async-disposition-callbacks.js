"use strict";

const { unwrapTransparentParent } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { memberPropertyName } = require("./async-targets");

function namedBinding(fn, context) {
  const value = unwrapTransparentParent(fn);
  const parent = value.parent;
  if (
    parent?.type === "VariableDeclarator" &&
    parent.init === value &&
    parent.id.type === "Identifier"
  ) {
    return {
      variable: resolveVariable(parent.id, context),
      stable: parent.parent.kind === "const",
    };
  }
  if (fn.type !== "FunctionDeclaration") return null;
  const variable = context.sourceCode
    .getDeclaredVariables(fn)
    .find((candidate) =>
      candidate.defs.some((def) => def.type === "FunctionName" && def.node === fn),
    );
  if (!variable) return null; // Anonymous default exports have no callback binding.
  return { variable, stable: !variable.references.some((reference) => reference.isWrite()) };
}

function nextBoundCall(value) {
  const member = value.parent;
  if (
    member?.type !== "MemberExpression" ||
    member.object !== value ||
    memberPropertyName(member) !== "bind"
  )
    return null;
  const callee = unwrapTransparentParent(member);
  const call = callee.parent;
  return call?.type === "CallExpression" && call.callee === callee
    ? unwrapTransparentParent(call)
    : null;
}

function boundResult(id) {
  let value = unwrapTransparentParent(id);
  for (;;) {
    const next = nextBoundCall(value);
    if (!next) return value;
    value = next;
  }
}

function argumentUse(id, result) {
  let value = result;
  while (value.parent) {
    const parent = value.parent;
    if (parent.type === "MemberExpression") return null;
    if (parent.type === "CallExpression") {
      if (parent.callee === value) return null;
      return { value, direct: value === unwrapTransparentParent(id) };
    }
    if (parent.type.endsWith("Statement") || parent.type === "VariableDeclarator") break;
    value = parent;
  }
  return null;
}

// null means this is an ordinary function return. A callback return needs a
// contract at every argument use, including uses reached through const aliases.
function namedCallbackDisposition(fn, context, acceptsArgument) {
  const binding = namedBinding(fn, context);
  if (!binding) return null;
  const seen = new Set();
  let passed = false;
  let accepted = true;

  function inspect(variable, stable) {
    if (seen.has(variable)) return;
    seen.add(variable);
    stable &&= !variable.references.some((reference) => reference.isWrite() && !reference.init);
    for (const reference of variable.references) {
      if (reference.init || reference.isWrite()) continue;
      const id = reference.identifier;
      const value = boundResult(id);
      const bound = value !== unwrapTransparentParent(id);
      const parent = value.parent;
      if (
        parent?.type === "VariableDeclarator" &&
        parent.init === value &&
        parent.id.type === "Identifier"
      ) {
        inspect(
          resolveVariable(parent.id, context),
          stable && !bound && parent.parent.kind === "const",
        );
        continue;
      }
      const use = argumentUse(id, value);
      if (!use) continue;
      passed = true;
      if (!stable || !use.direct || !acceptsArgument(use.value)) accepted = false;
    }
  }

  inspect(binding.variable, binding.stable);
  return passed ? accepted : null;
}

module.exports = { namedCallbackDisposition };
