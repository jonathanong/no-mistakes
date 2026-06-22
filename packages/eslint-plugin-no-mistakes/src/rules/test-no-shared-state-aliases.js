"use strict";

function resolveVariable(node, context) {
  let scope = context.sourceCode.getScope(node);
  while (scope) {
    const variable = scope.variables.find((candidate) => candidate.name === node.name);
    if (variable) return variable;
    scope = scope.upper;
  }
  return null;
}

function baseIdentifier(callee) {
  if (callee?.type === "Identifier") return callee;
  if (callee?.type === "CallExpression") return baseIdentifier(callee.callee);
  if (callee?.type === "MemberExpression") return baseIdentifier(callee.object);
  return null;
}

function createImportedTestAliases(context) {
  const importedTestCallees = new Map();
  const describeAliases = new Set(["describe"]);
  return {
    add(local, imported) {
      importedTestCallees.set(local.name, resolveVariable(local, context));
      if (imported === "describe") describeAliases.add(local.name);
    },
    isDescribeAliasCall(callee) {
      const base = baseIdentifier(callee);
      return (
        base?.type === "Identifier" && describeAliases.has(base.name) && !this.isShadowed(callee)
      );
    },
    isShadowed(callee) {
      const base = baseIdentifier(callee);
      const imported = base ? importedTestCallees.get(base.name) : null;
      return Boolean(imported && resolveVariable(base, context) !== imported);
    },
    hasProperty(callee, name) {
      if (hasOwnProperty(callee, name)) return true;
      if (callee?.type === "MemberExpression") return this.hasProperty(callee.object, name);
      if (callee?.type === "CallExpression") return this.hasProperty(callee.callee, name);
      return false;
    },
  };
}

function hasOwnProperty(callee, name) {
  return callee?.type === "MemberExpression" && !callee.computed && callee.property?.name === name;
}

module.exports = {
  createImportedTestAliases,
  hasOwnProperty,
  resolveVariable,
};
