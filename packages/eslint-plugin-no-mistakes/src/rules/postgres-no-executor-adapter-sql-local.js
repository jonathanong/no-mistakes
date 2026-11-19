"use strict";

const { resolveVariable, unwrapTs } = require("./postgres-query-text");
const { objectMutation } = require("./postgres-no-executor-adapter-object-mutation");

function parameterVariable(node, context) {
  const variable = node.type === "Identifier" ? resolveVariable(node, context) : null;
  return variable?.defs.length && variable.defs.every((def) => def.type === "Parameter")
    ? variable
    : null;
}

function executorParameter(node, context) {
  const variable = parameterVariable(node, context);
  const type = variable?.defs[0].name.typeAnnotation?.typeAnnotation;
  const names = context.options?.[0]?.executorTypeNames ?? [];
  return type?.type === "TSTypeReference" && type.typeName.type === "Identifier"
    ? names.includes(type.typeName.name)
    : false;
}

function constDeclarator(variable) {
  const def = variable.defs.length === 1 ? variable.defs[0] : null;
  return def?.type === "Variable" && def.parent.kind === "const" && def.node.init ? def.node : null;
}

// `const { rows } = await query(...)` where `query` is a typed executor parameter.
function executorResult(variable, context) {
  const declarator = constDeclarator(variable);
  if (declarator?.id.type !== "ObjectPattern") return false;
  const init = unwrapTs(declarator.init);
  const call = unwrapTs(init.type === "AwaitExpression" ? init.argument : init);
  return (
    call.type === "CallExpression" &&
    !call.optional &&
    executorParameter(unwrapTs(call.callee), context)
  );
}

// A never-reassigned const bound to a parameter, an element of one, or an element of
// a destructured executor result is the same receiver proof as the parameter itself.
function localReceiver(variable, context) {
  const declarator = constDeclarator(variable);
  if (declarator?.id.type !== "Identifier") return false;
  let root = unwrapTs(declarator.init);
  while (root.type === "MemberExpression" && !root.optional) root = unwrapTs(root.object);
  const rootVariable = root.type === "Identifier" ? resolveVariable(root, context) : null;
  return Boolean(
    rootVariable &&
    rootVariable !== variable &&
    (parameterVariable(root, context) || executorResult(rootVariable, context)) &&
    !objectMutation(rootVariable, null, null, root, context),
  );
}

module.exports = { localReceiver };
