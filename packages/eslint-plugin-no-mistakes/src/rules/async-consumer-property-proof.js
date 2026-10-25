"use strict";
const { unwrapTransparentParent } = require("./async-ast");
const { resolveVariable, memberPropertyName } = require("./async-target-bindings");
const { localBodySafe } = require("./async-consumer-local-proof");

function propertyBodyAwaited(fn, index, context, property, proveCall, receiverShape, shape) {
  const parameter = fn?.params[index];
  const param = parameter?.type === "AssignmentPattern" ? parameter.left : parameter;
  if (
    param?.type !== "Identifier" ||
    !localBodySafe(fn, context) ||
    fn.params.some((item) => item.type === "RestElement")
  )
    return false;
  const variable = resolveVariable(param, context);
  let count = 0;
  const safe = variable.references
    .filter((ref) => !ref.init)
    .every((ref) => {
      if (ref.isWrite()) return false;
      const value = unwrapTransparentParent(ref.identifier);
      const member = value.parent;
      if (member?.type === "MemberExpression" && member.object === value) {
        const name = memberPropertyName(member);
        if (
          name === null ||
          member.optional ||
          ["__proto__", "constructor", "prototype"].includes(name)
        )
          return false;
        // Other statically named fields do not change this callback's disposition.
        if (name !== property) {
          const parent = unwrapTransparentParent(member).parent;
          return (
            !(
              parent?.type === "CallExpression" && parent.callee === unwrapTransparentParent(member)
            ) ||
            shape.get(name) === true ||
            (parent.optional && !shape.has(name))
          );
        }
        count++;
        if (!lexicalCallback(member, fn)) return false;
        let selected = unwrapTransparentParent(member);
        if (
          selected.parent?.type === "LogicalExpression" &&
          selected.parent.operator === "??" &&
          selected.parent.left === selected
        )
          selected = unwrapTransparentParent(selected.parent);
        const invocation = selected.parent;
        if (invocation?.type === "CallExpression" && invocation.callee === selected)
          return !invocation.optional && awaited(invocation);
        return forwarded(selected, null, context, proveCall, receiverShape, variable, shape);
      }
      count++;
      return forwarded(value, property, context, proveCall, receiverShape, variable, shape);
    });
  return safe && count > 0;
}

// Inspect lexical callback bodies passed directly to a call. A stored or returned
// closure is an escape, so merely containing an await cannot establish adoption.
function lexicalCallback(node, fn) {
  for (let owner = node.parent; owner && owner !== fn; owner = owner.parent) {
    if (
      !["ArrowFunctionExpression", "FunctionExpression", "FunctionDeclaration"].includes(owner.type)
    )
      continue;
    let value = unwrapTransparentParent(owner);
    if (
      value.parent?.type === "Property" &&
      value.parent.value === value &&
      value.parent.parent.type === "ObjectExpression"
    )
      value = unwrapTransparentParent(value.parent.parent);
    if (value.parent?.type !== "CallExpression" || !value.parent.arguments.includes(value))
      return false;
  }
  return true;
}

function awaited(call) {
  const value = unwrapTransparentParent(call);
  return value.parent?.type === "AwaitExpression" && value.parent.argument === value;
}

function forwarded(value, property, context, proveCall, receiverShape, variable, shape) {
  let argument = value;
  if (value.parent?.type === "SpreadElement") {
    const spread = value.parent;
    const object = spread.parent;
    if (
      object.type !== "ObjectExpression" ||
      object.properties.some(
        (entry) =>
          entry !== spread &&
          (entry.type !== "Property" ||
            entry.computed ||
            entry.method ||
            entry.kind !== "init" ||
            (entry.key.name ?? entry.key.value) === property),
      )
    )
      return false;
    shape = receiverShape(object, context, variable, shape);
    argument = unwrapTransparentParent(object);
  } else if (property === null && value.parent?.type === "Property") {
    const entry = value.parent;
    const object = entry.parent;
    if (
      entry.value !== value ||
      object.type !== "ObjectExpression" ||
      object.properties.some(
        (item) => item.type !== "Property" || item.computed || item.method || item.kind !== "init",
      )
    )
      return false;
    const names = object.properties.map((item) => item.key.name ?? item.key.value);
    if (new Set(names).size !== names.length) return false;
    property = entry.key.name ?? entry.key.value;
    shape = receiverShape(object, context, variable, shape);
    argument = unwrapTransparentParent(object);
  }
  const call = argument.parent;
  if (call?.type !== "CallExpression" || !awaited(call)) return false;
  const index = call.arguments.indexOf(argument);
  return index >= 0 && proveCall(call, index, property, context, shape);
}

module.exports = { propertyBodyAwaited };
