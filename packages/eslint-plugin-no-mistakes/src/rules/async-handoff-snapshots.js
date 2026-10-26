"use strict";
const { unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");

// A parameter can coerce arbitrary objects. Its earlier numeric snapshot cannot
// invoke a supplied const callback that has not been created in this block yet.
function beforeCallbacks(call, context, callbacks) {
  const argument = unwrapExpression(call.arguments[0]);
  const binding = argument?.type === "Identifier" && resolveVariable(argument, context);
  if (
    call.arguments.length !== 1 ||
    !binding ||
    binding.defs.length !== 1 ||
    binding.defs[0].type !== "Parameter" ||
    !callbacks.size
  )
    return false;
  const statement = call.parent.type === "VariableDeclarator" && call.parent.parent;
  if (!statement || statement.kind !== "const" || statement.parent.type !== "BlockStatement")
    return false;
  return [...callbacks].every((callback) => {
    const owner = callback.parent;
    const creation = owner.parent;
    return (
      owner.type === "VariableDeclarator" &&
      creation.kind === "const" &&
      creation.parent === statement.parent &&
      statement.range[1] < creation.range[0]
    );
  });
}

module.exports = { beforeCallbacks };
