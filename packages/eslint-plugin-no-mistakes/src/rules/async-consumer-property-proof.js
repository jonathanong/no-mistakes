"use strict";
const { unwrapTransparentParent } = require("./async-ast");
const { adoptNode, intersectShapes } = require("./async-consumer-property-carrier");
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
  const seen = new WeakSet();
  let count = 0;
  // null until a use adopts the callback or returns the copied object.
  // Mixed results fail the walk instead of combining into one outcome.
  let outcome = null;
  const note = (mode) => {
    if (mode === true) {
      if (outcome?.returned) return false;
      outcome = true;
      return true;
    }
    if (mode?.returned) {
      if (outcome === true) return false;
      outcome = outcome
        ? { returned: true, shape: intersectShapes(outcome.shape, mode.shape) }
        : mode;
      return true;
    }
    return false;
  };
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
        // Runtime writes invalidate the supplied receiver shape, even for siblings.
        if (propertyWritten(member)) return false;
        // Pure reads of other static fields preserve this callback's disposition.
        if (name !== property) {
          const parent = unwrapTransparentParent(member).parent;
          // A chained access can invoke or mutate an unproven sibling method.
          if (parent?.type === "MemberExpression") return false;
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
          return note(!invocation.optional && awaited(invocation));
        return note(
          forwarded(selected, null, context, proveCall, receiverShape, variable, shape, seen),
        );
      }
      count++;
      return note(
        forwarded(value, property, context, proveCall, receiverShape, variable, shape, seen),
      );
    });
  // Every accepted use goes through note(), which records the outcome.
  if (!safe || !count) return false;
  return outcome;
}

// Follow member objects and assignment patterns only along their writable target.
function propertyWritten(member) {
  let target = unwrapTransparentParent(member);
  while (
    ["ObjectPattern", "ArrayPattern", "RestElement"].includes(target.parent.type) ||
    (target.parent.type === "MemberExpression" && target.parent.object === target) ||
    (target.parent.type === "Property" && target.parent.value === target) ||
    (target.parent.type === "AssignmentPattern" && target.parent.left === target)
  )
    target = unwrapTransparentParent(target.parent);
  const parent = target.parent;
  return (
    (["AssignmentExpression", "ForOfStatement", "ForInStatement"].includes(parent.type) &&
      parent.left === target) ||
    parent.type === "UpdateExpression" ||
    (parent.type === "UnaryExpression" && parent.operator === "delete")
  );
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

function forwarded(
  value,
  property,
  context,
  proveCall,
  receiverShape,
  variable,
  shape,
  seen = new WeakSet(),
) {
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
  return adoptNode(
    argument,
    property,
    context,
    shape,
    { proveCall, receiverShape, propertyWritten, forward: forwarded, variable },
    seen,
  );
}

module.exports = { propertyBodyAwaited, propertyWritten, forwarded };
