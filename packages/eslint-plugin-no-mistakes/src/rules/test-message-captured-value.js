"use strict";

const { resolveVariable } = require("./test-no-shared-state-aliases");

function unwrapCapturedValue(node) {
  let current = node;
  while (current) {
    if (["ChainExpression", "TSNonNullExpression"].includes(current.type)) {
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
  }
  return true;
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
