"use strict";

const { unwrapExpression, unwrapTransparentParent } = require("./async-ast");
const { staticKeyName } = require("./postgres-no-async-sql-builder-types");

function classReceiver(node) {
  let child = node;
  for (let parent = node.parent; parent; parent = parent.parent) {
    if (parent.type === "FunctionDeclaration") return null;
    if (parent.type === "FunctionExpression") {
      return parent.parent?.type === "MethodDefinition" ? parent.parent : null;
    }
    if (parent.type === "StaticBlock") return parent;
    if (parent.type === "PropertyDefinition" && parent.value === child) return parent;
    if (parent.type === "ClassBody") return null;
    child = parent;
  }
  return null;
}

function memberEntry(index, node) {
  const isAccess = node.type === "MemberExpression";
  if (
    isAccess
      ? unwrapExpression(node.object)?.type !== "ThisExpression"
      : node.parent?.type !== "ClassBody"
  ) {
    return null;
  }
  const key = isAccess ? node.property : node.key;
  const privateName = key.type === "PrivateIdentifier";
  const name = privateName ? key.name : staticKeyName(node);
  if (name === null) return null;
  const receiver = isAccess ? classReceiver(node) : node;
  if (!receiver) return null;
  const mode = receiver.static || receiver.type === "StaticBlock" ? "static" : "instance";
  const id = `${mode}:${privateName ? "private" : "public"}:${name}`;
  const classBody = receiver.parent;
  if (!index.has(classBody)) index.set(classBody, new Map());
  const members = index.get(classBody);
  if (!members.has(id)) {
    members.set(id, { writes: 0, lastField: null, hasListener: false, usesAccessor: false });
  }
  return members.get(id);
}

function isMemberWrite(node) {
  let target = unwrapTransparentParent(node);
  while (target.parent) {
    const parent = target.parent;
    switch (parent.type) {
      case "AssignmentExpression":
      case "ForInStatement":
      case "ForOfStatement":
        return parent.left === target;
      case "UpdateExpression":
        return true;
      case "UnaryExpression":
        return parent.operator === "delete";
      case "ArrayPattern":
      case "ObjectPattern":
      case "RestElement":
        break;
      case "AssignmentPattern":
        if (parent.left !== target) return false;
        break;
      case "Property":
        if (parent.parent?.type !== "ObjectPattern" || parent.value !== target) return false;
        break;
      default:
        return false;
    }
    target = parent;
  }
  return false;
}

module.exports = { isMemberWrite, memberEntry };
