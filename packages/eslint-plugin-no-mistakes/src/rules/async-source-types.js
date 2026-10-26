"use strict";

const { resolveVariable, stableConstInitializer } = require("./async-target-bindings");
const { createDeclarations } = require("./async-source-declarations");
const { compileTargets, targetMatches } = require("./async-patterns");

function annotation(node) {
  return node?.typeAnnotation?.typeAnnotation || null;
}

function createSourceTypes(context, sources) {
  const declarations = createDeclarations(sources);
  const targets = compileTargets(context.options?.[0] || {}, "targets");
  function fresh(node, seen) {
    return node && seen.size < 32 && !seen.has(node) ? new Set(seen).add(node) : null;
  }
  function builtin(id, owner) {
    return id?.type === "Identifier" && !resolveVariable(id, owner)?.defs.length;
  }
  function promise(fn, owner) {
    if (fn.generator) return false;
    if (fn.async) return true;
    const type = fn.returnType?.typeAnnotation;
    return (
      type?.type === "TSTypeReference" &&
      type.typeName.name === "Promise" &&
      builtin(type.typeName, owner)
    );
  }
  function target(id, owner, seen = new Set()) {
    const next = fresh(id, seen);
    if (!next || id.type !== "Identifier") return null;
    const imported = declarations.imported(id, owner);
    if (imported && targetMatches(targets, imported.source, imported.name)) {
      const resolved = declarations.callable(id, owner);
      return resolved && promise(resolved.node, resolved.context) ? resolved.node : null;
    }
    const init = stableConstInitializer(id, owner, new Set());
    return init ? target(init, owner, next) : null;
  }
  function targetType(type, owner, seen = new Set()) {
    const next = fresh(type, seen);
    if (!next) return null;
    if (type.type === "TSTypeQuery") return target(type.exprName, owner);
    if (type.type === "TSTypeAliasDeclaration") return targetType(type.typeAnnotation, owner, next);
    if (type.type === "TSTypeReference") {
      const declaration = declarations.declaration(type.typeName, owner);
      return declaration && targetType(declaration.node, declaration.context, next);
    }
    return null;
  }
  function property(type, owner, name, seen = new Set()) {
    const next = fresh(type, seen);
    if (!next) return null;
    if (type.type === "TSTypeAliasDeclaration")
      return property(type.typeAnnotation, owner, name, next);
    if (type.type === "TSTypeReference") {
      const args = type.typeArguments?.params || type.typeParameters?.params;
      if (
        ["Partial", "Required", "Readonly"].includes(type.typeName.name) &&
        builtin(type.typeName, owner) &&
        args?.length === 1
      )
        return property(args[0], owner, name, next);
      const declaration = declarations.declaration(type.typeName, owner);
      return declaration && property(declaration.node, declaration.context, name, next);
    }
    const members = type.type === "TSTypeLiteral" ? type.members : type.body?.body;
    if (!members) return null;
    const matches = members.filter(
      (item) =>
        item.type === "TSPropertySignature" &&
        !item.computed &&
        (item.key.name ?? item.key.value) === name,
    );
    // Extending interfaces and merged declarations require a fuller type model.
    if (matches.length !== 1 || type.extends?.length) return null;
    return { type: annotation(matches[0]), context: owner };
  }
  function valueType(value, owner, seen = new Set()) {
    const next = fresh(value, seen);
    if (!next) return null;
    if (value.type === "Identifier") {
      const variable = resolveVariable(value, owner);
      if (
        variable?.defs.length !== 1 ||
        variable.references.some((ref) => ref.isWrite() && !ref.init)
      )
        return null;
      const def = variable.defs[0];
      const type = annotation(def.name);
      if (type) return { type, context: owner };
      const init = stableConstInitializer(value, owner, new Set());
      return init ? valueType(init, owner, next) : null;
    }
    if (value.type === "MemberExpression" && !value.computed) {
      const receiver = valueType(value.object, owner, next);
      return receiver && property(receiver.type, receiver.context, value.property.name);
    }
    return null;
  }
  return { ...declarations, target, targetType, property, valueType };
}

module.exports = { annotation, createSourceTypes };
