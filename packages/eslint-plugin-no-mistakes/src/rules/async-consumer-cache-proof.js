"use strict";

const { resolveVariable } = require("./async-target-bindings");
const { propertiesWritten } = require("./async-disposition-mocks");

function sameKey(first, key, context) {
  if (key?.type === "Literal")
    return typeof key.value === "string" && first?.type === "Literal" && first.value === key.value;
  return (
    first?.type === "Identifier" &&
    key?.type === "Identifier" &&
    resolveVariable(first, context) === resolveVariable(key, context)
  );
}

function stableKey(key, fn, param, context) {
  if (key?.type === "Literal") return typeof key.value === "string";
  if (key?.type !== "Identifier") return false;
  const variable = resolveVariable(key, context);
  return (
    fn.params.some((item) => item !== param && resolveVariable(item, context) === variable) &&
    !variable.references.some((ref) => ref.isWrite())
  );
}

function builtinMap(node, context) {
  if (resolveVariable(node, context)?.defs.length || propertiesWritten(node, context)) return false;
  // Global-holder writes can replace Map without mentioning its bare identifier.
  const references = context.sourceCode.scopeManager.scopes.flatMap((scope) => scope.references);
  const binding = resolveVariable(node, context);
  // Do not infer built-in semantics after the constructor/prototype escapes to
  // reflective APIs or unknown consumers, whose effects are not modeled here.
  if (
    references.some(
      (ref) =>
        ref.identifier.name === "Map" &&
        resolveVariable(ref.identifier, context) === binding &&
        !(ref.isTypeReference && !ref.isValueReference) &&
        !(
          ref.identifier.parent.type === "NewExpression" &&
          ref.identifier.parent.callee === ref.identifier
        ),
    )
  )
    return false;
  const holders = ["globalThis", "global", "window", "self"];
  return !references.some((ref) => {
    if (!holders.includes(ref.identifier.name)) return false;
    if (propertiesWritten(ref.identifier, context)) return true;
    const member = ref.identifier.parent;
    if (member.type !== "MemberExpression" || member.object !== ref.identifier) return true;
    const name = member.computed ? member.property.value : member.property.name;
    if (name !== "Map" && typeof name === "string") return false;
    return member.parent.type !== "NewExpression" || member.parent.callee !== member;
  });
}

function mapCall(node, method, cache, key, context, count) {
  const member = node?.callee;
  const first = node?.arguments?.[0];
  return (
    node?.type === "CallExpression" &&
    !node.optional &&
    node.arguments.length === count &&
    member?.type === "MemberExpression" &&
    !member.computed &&
    !member.optional &&
    member.property.name === method &&
    member.object.type === "Identifier" &&
    resolveVariable(member.object, context) === cache &&
    sameKey(first, key, context)
  );
}

// This is deliberately a complete body grammar: any intervening effect, alias,
// escape, overwrite, early return, or exception may strand the stored promise.
function cachedBodyAwaited(fn, param, context) {
  if (!fn.async || fn.generator || fn.params.some((item) => item.type !== "Identifier"))
    return false;
  const body = fn.body?.type === "BlockStatement" ? fn.body.body : [];
  if (body.length !== 3) return false;
  const [declaration, guard, returned] = body;
  const entry = declaration.declarations?.[0];
  const init = entry?.init;
  if (
    declaration.type !== "VariableDeclaration" ||
    declaration.kind !== "const" ||
    declaration.declarations.length !== 1 ||
    entry.id.type !== "Identifier" ||
    init?.type !== "NewExpression" ||
    init.callee.type !== "Identifier" ||
    init.callee.name !== "Map" ||
    init.arguments.length !== 0 ||
    !builtinMap(init.callee, context)
  )
    return false;
  const cache = resolveVariable(entry.id, context);
  const test = guard.test;
  const inserted =
    guard.consequent?.type === "BlockStatement"
      ? guard.consequent.body.length === 1 && guard.consequent.body[0]
      : guard.consequent;
  const set = inserted?.expression;
  const key = set?.arguments?.[0];
  if (
    !stableKey(key, fn, param, context) ||
    guard.type !== "IfStatement" ||
    guard.alternate ||
    test?.type !== "UnaryExpression" ||
    test.operator !== "!" ||
    inserted?.type !== "ExpressionStatement" ||
    !mapCall(test.argument, "has", cache, key, context, 1) ||
    !mapCall(set, "set", cache, key, context, 2) ||
    returned.type !== "ReturnStatement" ||
    returned.argument?.type !== "AwaitExpression" ||
    !mapCall(returned.argument.argument, "get", cache, key, context, 1)
  )
    return false;
  const invocation = set.arguments[1];
  const parameter = resolveVariable(param, context);
  const reads = parameter?.references.filter((ref) => !ref.init) || [];
  return (
    invocation?.type === "CallExpression" &&
    !invocation.optional &&
    invocation.arguments.length === 0 &&
    invocation.callee.type === "Identifier" &&
    resolveVariable(invocation.callee, context) === parameter &&
    reads.length === 1 &&
    !reads[0].isWrite() &&
    reads[0].identifier === invocation.callee
  );
}

module.exports = { cachedBodyAwaited };
