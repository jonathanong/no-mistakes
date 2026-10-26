"use strict";
const { unwrapTransparentParent } = require("./async-ast");
const { resolveVariable, memberPropertyName } = require("./async-target-bindings");

// Reading an unsafe sibling into a const does not make a later hidden callback
// invocation safe. Primitive fields can still be copied into ordinary carriers.
function siblingRead(member, context, shape, proof) {
  const name = memberPropertyName(member);
  if (proof.receiverShape.dataFacts?.safeRead(member, context, shape)) return true;
  if (unwrapTransparentParent(member).parent?.type === "MemberExpression") return false;
  if (shape.get(name) === true) return true;
  const type = proof.valueType(member, context)?.type;
  if (
    ["TSStringKeyword", "TSNumberKeyword", "TSBooleanKeyword", "TSLiteralType"].includes(type?.type)
  )
    return true;
  function safe(node, seen) {
    const value = unwrapTransparentParent(node);
    const parent = value.parent;
    if (parent?.type === "MemberExpression") return false;
    if (parent?.type === "CallExpression")
      return parent.callee === value && parent.optional && !shape.has(name);
    if (parent?.type === "VariableDeclarator" && parent.init === value) {
      if (parent.id.type !== "Identifier" || parent.parent.kind !== "const") return false;
      const binding = resolveVariable(parent.id, context);
      if (!binding || seen.has(binding) || seen.size >= 32) return false;
      seen = new Set(seen).add(binding);
      return binding.references.every(
        (ref) => ref.init || (!ref.isWrite() && safe(ref.identifier, seen)),
      );
    }
    if (parent?.type === "Property" && parent.value === value) return safe(parent.parent, seen);
    return (
      parent?.type === "ExpressionStatement" ||
      (parent?.type === "UnaryExpression" && ["void", "typeof"].includes(parent.operator))
    );
  }
  return safe(member, new Set());
}

module.exports = { siblingRead };
