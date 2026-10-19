"use strict";

const { isFunction, unwrapExpression } = require("./async-ast");
const {
  interfacePropertyContract,
  valueContract,
} = require("./postgres-no-async-sql-builder-property");
const {
  annotationContract,
  combine,
  lookup,
  staticKeyName,
} = require("./postgres-no-async-sql-builder-types");

function functionValueContract(func, types) {
  if (!isFunction(func)) {
    return "unknown";
  }
  if (func.async) return "requires";
  return annotationContract(func.returnType, types);
}

function findClassMember(classNode, name) {
  return classNode.body.body.find(
    (member) =>
      (member.type === "MethodDefinition" || member.type === "PropertyDefinition") &&
      staticKeyName(member) === name,
  );
}

function baseMethodContract(classNode, name, types, seen) {
  if (!classNode?.superClass) return "missing";
  if (seen.has(classNode)) {
    return "unknown";
  }
  seen.add(classNode);
  const base = lookup(types.classes, unwrapExpression(classNode.superClass)?.name);
  if (!base) return "unknown";
  const method = findClassMember(base, name);
  if (!method) return baseMethodContract(base, name, types, seen);
  return method.typeAnnotation
    ? valueContract(method.typeAnnotation, types)
    : functionValueContract(method.value, types);
}

function mentionsClassValue(node, name, seen = new Set()) {
  if (!node || typeof node !== "object" || seen.has(node)) return false;
  seen.add(node);
  if (node.type === "Identifier" && node.name === name) {
    const parent = node.parent;
    const property =
      parent?.type === "MemberExpression" && parent.property === node && !parent.computed;
    return !property;
  }
  for (const [key, value] of Object.entries(node)) {
    if (key === "parent" || !value || typeof value !== "object") continue;
    if (Array.isArray(value)) {
      if (value.some((child) => mentionsClassValue(child, name, seen))) return true;
    } else if (mentionsClassValue(value, name, seen)) return true;
  }
  return false;
}

function extendsClass(classNode, target, types, seen) {
  if (!classNode?.superClass || seen.has(classNode)) return "no";
  seen.add(classNode);
  const superExpr = unwrapExpression(classNode.superClass);
  const targetName = target.id?.name;
  if (superExpr?.type !== "Identifier")
    return targetName && mentionsClassValue(superExpr, targetName) ? "unknown" : "no";
  const found = types.classes.get(superExpr.name);
  if (!found) return "no";
  if (found.ambiguous) return targetName === superExpr.name ? "unknown" : "no";
  if (found.value === target) return "yes";
  return extendsClass(found.value, target, types, seen);
}

function subclassContract(classNode, methodName, types) {
  if (!classNode?.id?.name) return "allows";
  let unknown = false;
  for (const candidate of types.classList) {
    if (candidate === classNode) continue;
    const relation = extendsClass(candidate, classNode, types, new Set());
    if (relation === "no") continue;
    if (relation === "unknown") {
      unknown = true;
      continue;
    }
    if (findClassMember(candidate, methodName)) {
      return "requires";
    }
  }
  return unknown ? "unknown" : "allows";
}

function heritageContract(func, types) {
  const method = func.parent;
  if (method?.type !== "MethodDefinition" && method?.type !== "PropertyDefinition") return "allows";
  const classNode = method.parent.parent;
  const name = staticKeyName(method);
  if (!name) return "unknown";
  const results = [];
  if (method.type === "PropertyDefinition" && method.typeAnnotation) {
    results.push(valueContract(method.typeAnnotation, types));
  }
  const base = baseMethodContract(classNode, name, types, new Set());
  if (base === "missing") results.push(method.override ? "unknown" : "allows");
  else results.push(base);
  // A same-file subclass that declares this method still has to match the
  // async signature. Removing `async` from the base alone is not safe.
  results.push(subclassContract(classNode, name, types));
  for (const implemented of classNode.implements || []) {
    const face = lookup(types.interfaces, unwrapExpression(implemented.expression)?.name);
    if (!face) {
      results.push("unknown");
      continue;
    }
    const property = interfacePropertyContract(face, name, types, new Set());
    results.push(property === "missing" ? "allows" : property);
  }
  return combine(results);
}

module.exports = {
  extendsClass,
  findClassMember,
  heritageContract,
};
