"use strict";

const { findContainingFunction, unwrapTransparentParent } = require("./async-ast");
const { memberPropertyName, resolveVariable } = require("./async-target-bindings");

const hiddenProperties = new Set(["__proto__", "constructor", "prototype"]);

function intersectShapes(left, right) {
  const names = new Set([...left.keys(), ...right.keys()]);
  const result = new Map();
  for (const name of names) result.set(name, left.get(name) === true && right.get(name) === true);
  return result;
}

function awaited(call) {
  const value = unwrapTransparentParent(call);
  return value.parent?.type === "AwaitExpression" && value.parent.argument === value;
}

// Any write through the binding replaces the copied callback or its receiver.
// `variable` is the binding followConst already resolved.
function bindingMutated(variable, propertyWritten) {
  if (variable.defs.length !== 1) return true;
  return variable.references.some((ref) => {
    if (ref.init) return false;
    if (ref.isWrite()) return true;
    const object = unwrapTransparentParent(ref.identifier);
    const parent = object.parent;
    return (
      parent?.type === "MemberExpression" && parent.object === object && propertyWritten(parent)
    );
  });
}

function followConst(id, property, context, shape, api, seen) {
  const variable = resolveVariable(id, context);
  if (!variable || seen.has(variable) || bindingMutated(variable, api.propertyWritten))
    return false;
  seen.add(variable);
  const reads = variable.references.filter((ref) => !ref.init);
  if (reads.length === 0) return false;
  // A stored or returned closure can await the copy without the caller ever running it.
  const owner = findContainingFunction(id);
  let adopted = false;
  let returned = null;
  for (const ref of reads) {
    if (!api.lexicalCallback(ref.identifier, owner)) return false;
    const mode = api.forward(
      unwrapTransparentParent(ref.identifier),
      property,
      context,
      api.proveCall,
      api.receiverShape,
      api.variable,
      shape,
      seen,
    );
    if (mode === "safe") continue;
    if (mode === true) {
      if (returned) return false;
      adopted = true;
    } else if (mode?.returned) {
      if (adopted) return false;
      returned = returned ? intersectShapes(returned, mode.shape) : mode.shape;
    } else return false;
  }
  if (!adopted && !returned) return false;
  return returned ? { returned: true, shape: returned } : true;
}

function siblingUse(member, property, shape) {
  if (property === null) return false;
  const name = memberPropertyName(member);
  if (name === null || member.optional || hiddenProperties.has(name)) return false;
  if (name !== property) {
    const selected = unwrapTransparentParent(member);
    const next = selected.parent;
    if (next?.type === "MemberExpression") return false;
    if (next?.type === "CallExpression" && next.callee === selected)
      return shape.get(name) === true || (next.optional && !shape.has(name)) ? "safe" : false;
    return "safe";
  }
  const selected = unwrapTransparentParent(member);
  const call = selected.parent;
  if (
    call?.type !== "CallExpression" ||
    call.callee !== selected ||
    call.optional ||
    !awaited(call)
  )
    return false;
  return true;
}

function adoptNode(node, property, context, shape, api, seen) {
  const value = unwrapTransparentParent(node);
  const parent = value.parent;
  if (!parent) return false;
  if (parent.type === "CallExpression" && parent.callee === value) {
    if (property !== null || parent.optional || !awaited(parent)) return false;
    return true;
  }
  if (parent.type === "MemberExpression" && parent.object === value)
    return siblingUse(parent, property, shape);
  if (parent.type === "CallExpression" && parent.arguments.includes(value)) {
    if (parent.optional) return false;
    const mode = api.proveCall(parent, parent.arguments.indexOf(value), property, context, shape);
    if (mode === true) return awaited(parent);
    if (mode?.returned) return adoptReturned(parent, mode, property, context, api, seen);
    return false;
  }
  if (
    parent.type === "VariableDeclarator" &&
    parent.init === value &&
    parent.id.type === "Identifier" &&
    parent.parent?.type === "VariableDeclaration" &&
    parent.parent.kind === "const"
  )
    return followConst(parent.id, property, context, shape, api, seen);
  if (parent.type === "ReturnStatement" && parent.argument === value)
    return { returned: true, shape };
  return false;
}

// A helper that returns the copied object adopts nothing by itself. The caller
// must await the property, or pass that object to an awaited consumer that does.
function adoptReturned(call, mode, property, context, api, seen = new WeakSet()) {
  let value = unwrapTransparentParent(call);
  if (mode.async) {
    if (!awaited(value)) return false;
    value = unwrapTransparentParent(value.parent);
  }
  return adoptNode(value, property, context, mode.shape, api, seen);
}

module.exports = { adoptNode, adoptReturned, followConst, intersectShapes };
