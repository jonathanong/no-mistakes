"use strict";

const { resolveVariable, unwrapTs } = require("./postgres-query-text");
const { executorOptionDefaults } = require("./postgres-executor");
const { matchesConfiguredModule } = require("./postgres-scoped-executors");
const { targetFactsFor } = require("./postgres-no-executor-adapter-facts");
const { loopIterable } = require("./postgres-no-executor-adapter-sql-loop");
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
  const names = executorOptionDefaults(context.options[0]).executorTypeNames;
  return type?.type === "TSTypeReference" && type.typeName.type === "Identifier"
    ? names.includes(type.typeName.name)
    : false;
}

function constDeclarator(variable, kinds = ["const"]) {
  const def = variable.defs.length === 1 ? variable.defs[0] : null;
  return def?.type === "Variable" && kinds.includes(def.parent.kind) && def.node.init
    ? def.node
    : null;
}

function factoryCall(init, context) {
  const call = init.type === "AwaitExpression" ? unwrapTs(init.argument) : null;
  const callee = call?.type === "CallExpression" ? unwrapTs(call.callee) : null;
  const def = callee?.type === "Identifier" ? resolveVariable(callee, context)?.defs[0] : null;
  const options = executorOptionDefaults(context.options[0]);
  return Boolean(
    def?.type === "ImportBinding" &&
    options.executorFactoryNames.includes(def.node.imported?.name) &&
    matchesConfiguredModule(def.parent.source.value, options.importSpecifier, context.filename),
  );
}

// `const tx = await beginTransaction()` / `await using tx = ...` with a configured factory.
function executorFactoryLocal(node, context) {
  const variable = node.type === "Identifier" ? resolveVariable(node, context) : null;
  const declarator = variable && constDeclarator(variable, ["const", "await using"]);
  return Boolean(
    declarator?.id.type === "Identifier" &&
    factoryCall(declarator.init, context) &&
    !objectMutation(variable, null, null, node, context, node),
  );
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
    (executorParameter(unwrapTs(call.callee), context) ||
      executorFactoryLocal(unwrapTs(call.callee), context))
  );
}

// `await helper(tx, ...)` where `tx` is a typed executor parameter and the helper is proven.
function helperResult(init, context) {
  const call = init.type === "AwaitExpression" ? unwrapTs(init.argument) : null;
  if (call?.type !== "CallExpression" || call.optional) return false;
  const index = call.arguments.findIndex((arg) => executorParameter(unwrapTs(arg), context));
  if (index < 0) return false;
  const options = executorOptionDefaults(context.options[0]);
  const { provenHelper } = require("./postgres-no-executor-adapter-hop");
  return provenHelper(call, index, context, options, targetFactsFor(context));
}

function receiverInit(variable) {
  const declarator = constDeclarator(variable);
  return declarator?.id.type === "Identifier" ? declarator.init : loopIterable(variable.defs[0]);
}

// A never-reassigned const bound to a parameter, an element of one, or an element of
// a destructured executor result is the same receiver proof as the parameter itself.
// So is a `for...of` element of one, or the awaited result of a proven helper.
function localReceiver(variable, context, before = null) {
  const init = receiverInit(variable);
  if (!init) return false;
  let root = unwrapTs(init);
  if (helperResult(root, context)) return true;
  while (root.type === "MemberExpression" && !root.optional) root = unwrapTs(root.object);
  const rootVariable = root.type === "Identifier" ? resolveVariable(root, context) : null;
  return Boolean(
    rootVariable &&
    rootVariable !== variable &&
    (parameterVariable(root, context) || executorResult(rootVariable, context)) &&
    !objectMutation(rootVariable, null, null, root, context, before),
  );
}

module.exports = { localReceiver };
