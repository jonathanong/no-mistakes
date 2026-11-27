"use strict";

const { unwrapExpression } = require("./async-ast");

function isDynamicScope(scope, evalScopes) {
  return scope.type === "with" || scope.directCallToEvalScope || evalScopes.has(scope);
}

function readsRequireNamespace(node, sourceCode) {
  const expression = unwrapExpression(node);
  if (expression?.type === "MemberExpression") {
    return readsRequireNamespace(expression.object, sourceCode);
  }
  if (
    expression?.type !== "CallExpression" ||
    expression.callee?.type !== "Identifier" ||
    expression.callee.name !== "require"
  ) {
    return false;
  }
  for (let scope = sourceCode.getScope(expression.callee); scope; scope = scope.upper) {
    if (scope.set.get("require")?.defs.length) return false;
  }
  return true;
}

// See docs/eslint-rules/no-locale-less-date-format.md for the module/local split.
function isUnreadModuleDef(def, sourceCode) {
  const node = def.node;
  if (node?.type === "TSImportEqualsDeclaration") {
    return node.moduleReference?.type === "TSExternalModuleReference";
  }
  if (def.type === "ImportBinding") return true;
  return (
    def.type === "Variable" &&
    node?.type === "VariableDeclarator" &&
    readsRequireNamespace(node.init, sourceCode)
  );
}

function globalResolution(identifier, sourceCode, evalScopes) {
  let reference;
  for (let scope = sourceCode.getScope(identifier); scope; scope = scope.upper) {
    if (isDynamicScope(scope, evalScopes)) return "unknown";
    reference ??= scope.references.find((item) => item.identifier === identifier);
  }
  const variable = reference?.resolved;
  if (variable?.scope?.type === "global" && variable.defs.length === 0) return "global";
  if (
    variable?.defs.length &&
    !variable.references.some((reference) => reference.isWrite() && !reference.init) &&
    variable.defs.every((def) => isUnreadModuleDef(def, sourceCode))
  ) {
    return "module";
  }
  return "local";
}

module.exports = { globalResolution, isDynamicScope };
