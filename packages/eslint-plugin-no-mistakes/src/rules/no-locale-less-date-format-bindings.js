"use strict";

const { unwrapExpression } = require("./async-ast");

function isDynamicScope(scope, evalScopes) {
  return scope.type === "with" || scope.directCallToEvalScope || evalScopes.has(scope);
}

function isRequireCall(node) {
  const expression = unwrapExpression(node);
  return (
    expression?.type === "CallExpression" &&
    expression.callee?.type === "Identifier" &&
    expression.callee.name === "require"
  );
}

// An import, `import = require`, or `require()` names a module this file
// cannot read. `import =` of a local name, and a local object, stay quiet.
function isUnreadModuleDef(def) {
  const node = def.node;
  if (node?.type === "TSImportEqualsDeclaration") {
    return node.moduleReference?.type === "TSExternalModuleReference";
  }
  if (def.type === "ImportBinding") return true;
  return def.type === "Variable" && node?.type === "VariableDeclarator" && isRequireCall(node.init);
}

function globalResolution(identifier, sourceCode, evalScopes) {
  let reference;
  for (let scope = sourceCode.getScope(identifier); scope; scope = scope.upper) {
    if (isDynamicScope(scope, evalScopes)) return "unknown";
    reference ??= scope.references.find((item) => item.identifier === identifier);
  }
  const variable = reference?.resolved;
  if (variable?.scope?.type === "global" && variable.defs.length === 0) return "global";
  if (variable?.defs.length && variable.defs.every(isUnreadModuleDef)) return "module";
  return "local";
}

module.exports = { globalResolution, isDynamicScope };
