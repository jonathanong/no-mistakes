"use strict";

const { childNodes, resolveVariable, unwrapTs } = require("./postgres-query-text");

function opaqueScope(root) {
  const stack = [root];
  while (stack.length) {
    const node = stack.pop();
    if (node.type === "Identifier" && ["arguments", "eval"].includes(node.name)) return true;
    stack.push(...childNodes(node));
  }
  return false;
}

// Called only for the simple bound identifier already selected by localTarget.
// ESLint's prepared scope references preserve binding identity without a scan.
function localOptionBody(fn, bound, context, facts) {
  let cache = facts.get(fn);
  if (!cache) {
    cache = new Map();
    cache.opaqueScope = opaqueScope(fn.body);
    facts.set(fn, cache);
  }
  if (cache.has(bound)) return cache.get(bound);
  const variable = resolveVariable(bound, context);
  const safe = Boolean(variable && !cache.opaqueScope && safeVariable(variable, context));
  cache.set(bound, safe);
  return safe;
}

function safeVariable(variable, context) {
  for (const ref of variable.references) {
    if (ref.isWrite()) {
      if (ref.init) continue;
      return false;
    }
    const identifier = ref.identifier;
    const parent = identifier.parent;
    if (parent?.type === "TSTypeQuery" && parent.exprName === identifier) continue;
    if (parent?.type === "UnaryExpression" && parent.operator === "typeof") continue;
    if (!safeValue(identifier, context)) return false;
  }
  return true;
}

function safeValue(expression, context) {
  let parent = expression.parent;
  while (parent?.expression === expression && unwrapTs(parent) !== parent) {
    expression = parent;
    parent = expression.parent;
  }
  if (parent?.type === "CallExpression" && parent.callee === expression)
    return parent.arguments.every((argument) => argument.type !== "SpreadElement");
  if (judgedMethod(expression)) return true;
  if (parent?.type !== "VariableDeclarator" || parent.init !== expression) return false;
  if (parent.parent.kind !== "const" || parent.id.type !== "Identifier") return false;
  const alias = resolveVariable(parent.id, context);
  return Boolean(alias && safeVariable(alias, context));
}

function judgedMethod(node) {
  const member = node.parent;
  if (member?.type !== "MemberExpression" || member.object !== node || member.optional)
    return false;
  const name = member.computed ? member.property?.value : member.property?.name;
  if (name !== "call" && name !== "apply") return false;
  const call = member.parent;
  if (call?.type !== "CallExpression" || call.callee !== member) return false;
  if (call.arguments.some((argument) => argument.type === "SpreadElement")) return false;
  if (name === "call") return true;
  const array = call.arguments[1];
  return Boolean(
    array?.type === "ArrayExpression" &&
    array.elements.every((element) => element && element.type !== "SpreadElement"),
  );
}

module.exports = { localOptionBody };
