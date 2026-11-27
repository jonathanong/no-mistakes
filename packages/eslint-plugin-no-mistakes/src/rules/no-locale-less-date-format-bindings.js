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

function hasProvenLocalAssignment(variable, read) {
  const writes = variable.references.filter((reference) => reference.isWrite() && !reference.init);
  if (writes.length !== 1) return false;
  const [write] = writes;
  const assignment = write.identifier.parent;
  const statement = assignment.parent;
  return (
    write.from === read.from &&
    assignment.type === "AssignmentExpression" &&
    assignment.operator === "=" &&
    unwrapExpression(assignment.right).type === "ObjectExpression" &&
    statement.type === "ExpressionStatement" &&
    ["Program", "BlockStatement"].includes(statement.parent.type) &&
    statement.range[1] <= read.identifier.range[0]
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
    variable.defs.every((def) => isUnreadModuleDef(def, sourceCode)) &&
    !hasProvenLocalAssignment(variable, reference)
  ) {
    return "module";
  }
  return "local";
}

module.exports = { globalResolution, isDynamicScope };
