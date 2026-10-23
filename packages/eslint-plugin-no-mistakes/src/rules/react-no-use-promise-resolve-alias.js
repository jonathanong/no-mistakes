"use strict";

const { resolveVariable } = require("./test-no-shared-state-aliases");
const { literalString } = require("./module-mock-helpers");

function writesNamespaceMember(variable) {
  return variable.references.some((reference) => {
    const member = reference.identifier.parent;
    if (member?.type !== "MemberExpression" || member.object !== reference.identifier) return false;
    const parent = member.parent;
    return (
      (parent.type === "AssignmentExpression" && parent.left === member) ||
      (parent.type === "UpdateExpression" && parent.argument === member) ||
      (parent.type === "UnaryExpression" && parent.operator === "delete")
    );
  });
}

function aliasUsesStayLocal(variable, context, seen = new Set()) {
  if (seen.has(variable) || writesNamespaceMember(variable)) return false;
  seen.add(variable);
  return variable.references.every((reference) => {
    if (reference.init) return true;
    if (reference.isWrite() || crossesWith(reference.identifier, variable.scope, context))
      return false;
    const parent = reference.identifier.parent;
    if (parent?.type === "MemberExpression" && parent.object === reference.identifier) {
      const name = parent.computed ? literalString(parent.property) : parent.property.name;
      return (
        name === "resolve" &&
        parent.parent?.type === "CallExpression" &&
        parent.parent.callee === parent
      );
    }
    if (
      parent?.type !== "VariableDeclarator" ||
      parent.init !== reference.identifier ||
      parent.parent?.kind !== "const" ||
      parent.id.type !== "Identifier" ||
      parent.parent.parent?.type === "ExportNamedDeclaration" ||
      variable.defs[0].node.init.range[1] > parent.init.range[0]
    )
      return false;
    const child = resolveVariable(parent.id, context);
    return child?.defs.length === 1 && aliasUsesStayLocal(child, context, seen);
  });
}

function globalUsesStayLocal(variable, context) {
  return variable.references.every((reference) => {
    if (reference.isTypeReference && !reference.isValueReference) return true;
    if (reference.isWrite()) return false;
    const parent = reference.identifier.parent;
    if (parent?.type === "MemberExpression" && parent.object === reference.identifier) {
      const name = parent.computed ? literalString(parent.property) : parent.property.name;
      return (
        ["resolve", "reject", "all", "allSettled", "any", "race"].includes(name) &&
        parent.parent?.type === "CallExpression" &&
        parent.parent.callee === parent
      );
    }
    if (parent?.type === "NewExpression" && parent.callee === reference.identifier) return true;
    if (
      parent?.type !== "VariableDeclarator" ||
      parent.init !== reference.identifier ||
      parent.parent?.kind !== "const" ||
      parent.id.type !== "Identifier"
    )
      return false;
    const child = resolveVariable(parent.id, context);
    return child?.defs.length === 1 && aliasUsesStayLocal(child, context);
  });
}

function hasGlobalObjectPromiseAccess(context) {
  const globalScope = context.sourceCode.scopeManager.globalScope;
  for (const name of ["globalThis", "window", "self", "global"]) {
    const variable = globalScope.set.get(name);
    const references =
      variable && variable.defs.length === 0
        ? variable.references
        : globalScope.through.filter(
            (reference) => reference.identifier.name === name && !reference.resolved,
          );
    if (
      references.some((reference) => {
        const member = reference.identifier.parent;
        if (member?.type !== "MemberExpression" || member.object !== reference.identifier)
          return true; // The global object escaped or was used as a value.
        const property = member.computed ? literalString(member.property) : member.property.name;
        if (property === null || property === "Promise") return true;
        const parent = member.parent;
        return (
          (parent.type === "CallExpression" && parent.callee === member) ||
          (parent.type === "AssignmentExpression" && parent.left === member) ||
          (parent.type === "UpdateExpression" && parent.argument === member) ||
          (parent.type === "UnaryExpression" && parent.operator === "delete")
        );
      })
    )
      return true;
  }
  return false;
}

function crossesWith(identifier, bindingScope, context) {
  for (
    let scope = context.sourceCode.getScope(identifier);
    scope && scope !== bindingScope;
    scope = scope.upper
  ) {
    if (scope.type === "with") return true;
  }
  return false;
}

function immutableAlias(identifier, context) {
  const variable = resolveVariable(identifier, context);
  if (variable?.defs.length !== 1) return null;
  const [definition] = variable.defs;
  const declarator = definition.node;
  if (
    definition.type !== "Variable" ||
    declarator?.parent?.kind !== "const" ||
    declarator.id.type !== "Identifier" ||
    declarator.id.name !== identifier.name ||
    declarator.init?.type !== "Identifier" ||
    declarator.parent.parent?.type === "ExportNamedDeclaration" ||
    variable.references.some((reference) => reference.isWrite() && !reference.init) ||
    !aliasUsesStayLocal(variable, context) ||
    declarator.init.range[1] > identifier.range[0] ||
    crossesWith(identifier, variable.scope, context)
  )
    return null;
  return declarator;
}

function isGlobalPromiseAlias(identifier, context) {
  let current = identifier;
  while (true) {
    const declarator = immutableAlias(current, context);
    if (!declarator) return false;
    const next = declarator.init;
    const binding = resolveVariable(next, context);
    // A later const initializer is still in its TDZ at this read. This
    // decreasing source-order invariant also rules out alias cycles.
    if (binding?.defs.length && binding.defs[0].node?.init?.range[1] > next.range[0]) return false;
    if (next.name === "Promise") {
      return (
        binding?.defs.length === 0 &&
        binding.scope.type === "global" &&
        !writesNamespaceMember(binding) &&
        globalUsesStayLocal(binding, context) &&
        !hasGlobalObjectPromiseAccess(context) &&
        !crossesWith(next, binding?.scope, context)
      );
    }
    if (crossesWith(next, binding?.scope, context)) return false;
    current = next;
  }
}

module.exports = { isGlobalPromiseAlias };
