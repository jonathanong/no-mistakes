"use strict";

const { resolveVariable } = require("./test-no-shared-state-aliases");

function unwrapCapturedValue(node) {
  let current = node;
  while (current) {
    if (
      ["ChainExpression", "TSNonNullExpression", "TSSatisfiesExpression"].includes(current.type)
    ) {
      current = current.expression;
    } else if (current.type === "SequenceExpression") {
      current = current.expressions.at(-1);
    } else if (
      ["TSAsExpression", "TSTypeAssertion"].includes(current.type) &&
      current.typeAnnotation.type === "TSStringKeyword"
    ) {
      current = current.expression;
    } else return current;
  }
  return current;
}

function safeReference(identifier, variable, context) {
  for (
    let scope = context.sourceCode.getScope(identifier);
    scope !== variable.scope;
    scope = scope.upper
  ) {
    if (scope.type === "with") return false;
    // A hoisted declaration can run before the captured const is initialized,
    // even when the assertion appears textually after that initializer.
    if (scope.type === "function" && scope.block.type === "FunctionDeclaration") return false;
    // ESLint may ignore eval when building scopes. Non-strict direct eval can
    // introduce a nearer variable that the static resolver cannot see.
    if (hasDirectEval(scope)) return false;
  }
  return true;
}

function hasDirectEval(scope) {
  if (scope.isStrict) return false;
  if (scope.references.some(isDirectEvalReference)) return true;
  // A block's eval inserts vars into its containing function, unlike an eval
  // in a nested function with its own variable environment.
  return scope.childScopes.some(
    (child) => child.variableScope === scope.variableScope && hasDirectEval(child),
  );
}

function isDirectEvalReference({ identifier }) {
  if (identifier.name !== "eval") return false;
  let callee = identifier;
  while (
    [
      "ChainExpression",
      "TSAsExpression",
      "TSTypeAssertion",
      "TSNonNullExpression",
      "TSSatisfiesExpression",
    ].includes(callee.parent?.type)
  )
    callee = callee.parent;
  return (
    callee.parent?.type === "CallExpression" &&
    callee.parent.callee === callee &&
    !callee.parent.optional
  );
}

function capturedMessageOrigin(node, context) {
  let current = unwrapCapturedValue(node);
  let captured = false;
  while (current?.type === "Identifier") {
    const variable = resolveVariable(current, context);
    if (variable?.defs.length !== 1) return null;
    const [definition] = variable.defs;
    const declarator = definition.node;
    if (
      definition.type !== "Variable" ||
      declarator.parent.kind !== "const" ||
      declarator.id.type !== "Identifier" ||
      !declarator.init ||
      variable.references.some((reference) => reference.isWrite() && !reference.init) ||
      declarator.init.range[1] > current.range[0] ||
      !safeReference(current, variable, context)
    )
      return null;
    // Every alias hop moves to an earlier initializer, ruling out cycles.
    current = unwrapCapturedValue(declarator.init);
    captured = true;
  }
  return captured && current?.type === "MemberExpression" ? current : null;
}

module.exports = { capturedMessageOrigin };
