"use strict";

const { unwrapExpression } = require("./async-ast");

function findVariable(scope, name) {
  for (let current = scope; current; current = current.upper) {
    const variable =
      current.set?.get?.(name) ?? current.variables?.find((item) => item.name === name);
    if (variable) return variable;
  }
  return null;
}

function crossesWith(identifier, variable, context) {
  for (
    let scope = context.sourceCode.getScope(identifier);
    scope !== variable.scope;
    scope = scope.upper
  ) {
    if (scope.type === "with") return true;
  }
  return false;
}

function immutableDeclarator(identifier, context) {
  const variable = findVariable(context.sourceCode.getScope(identifier), identifier.name);
  if (variable?.defs.length !== 1) return null;
  const [definition] = variable.defs;
  if (definition.type !== "Variable") return null;
  const declarator = definition.node;
  if (
    !declarator?.init ||
    declarator.id.type !== "Identifier" ||
    declarator.id.name !== identifier.name ||
    declarator.parent?.type !== "VariableDeclaration" ||
    declarator.parent.kind !== "const"
  )
    return null;
  return { variable, declarator };
}

function originDeclarator(identifier, context) {
  let reference = identifier;
  while (true) {
    const binding = immutableDeclarator(reference, context);
    if (!binding) return null;
    const { declarator } = binding;
    const value = unwrapExpression(declarator.init);
    if (value.type !== "Identifier") return declarator;
    // A with-object getter can replace an outer lexical value at either read.
    if (crossesWith(reference, binding.variable, context)) return null;
    if (binding.variable.references.some((ref) => ref.isWrite() && !ref.init)) return null;
    // Reading an alias before initialization, or creating it from a binding
    // still in the TDZ, does not establish promise identity.
    if (declarator.init.range[1] > reference.range[0]) return null;
    const target = immutableDeclarator(value, context);
    // Each hop moves to an earlier initializer. That also rules out cycles.
    if (
      !target ||
      crossesWith(value, target.variable, context) ||
      target.variable.references.some((ref) => ref.isWrite() && !ref.init) ||
      target.declarator.init.range[1] > declarator.init.range[0]
    )
      return null;
    reference = value;
  }
}

module.exports = { originDeclarator };
