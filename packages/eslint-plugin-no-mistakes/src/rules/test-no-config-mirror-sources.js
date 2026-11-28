"use strict";

const { isFunction, unwrapExpression } = require("./async-ast");
const { memberPropertyName, resolveVariable } = require("./async-target-bindings");
const { isUnshadowedGlobal } = require("./test-no-config-mirror-bindings");
const { pathVariable } = require("./test-no-config-mirror-paths");
const { sourceDeclarationLineNode } = require("./test-no-config-mirror-scopes");

const READ_METHODS = new Set([
  "fs.readFileSync",
  "fs.readFile",
  "fs.promises.readFile",
  "fs/promises.readFile",
  "fs-extra.readFileSync",
  "fs-extra.readFile",
  "fs-extra.readJsonSync",
  "fs-extra.readJson",
]);
const BUILTIN_PARSERS = new Set([
  "yaml.parse",
  "yaml.parseDocument",
  "yaml.parseAllDocuments",
  "js-yaml.load",
  "js-yaml.loadAll",
]);

function isParserCallee(callee, resolve, configuredParsers, context) {
  if (
    callee?.type === "MemberExpression" &&
    memberPropertyName(callee) === "parse" &&
    isUnshadowedGlobal(unwrapExpression(callee.object), "JSON", context)
  ) {
    return true;
  }
  if (isUnshadowedGlobal(callee, "String", context)) return true;
  const binding = resolve(callee);
  return Boolean(binding && (BUILTIN_PARSERS.has(binding) || configuredParsers.has(binding)));
}

function isReadCall(callNode, resolve, promisifiedVars, context) {
  const binding = resolve(callNode.callee);
  if (binding && READ_METHODS.has(binding)) return { isCallback: binding === "fs.readFile" };
  const callee = unwrapExpression(callNode.callee);
  if (callee?.type === "Identifier" && promisifiedVars.has(resolveVariable(callee, context))) {
    return { isCallback: false };
  }
  return null;
}

function readHelperInfo(fn, resolve, promisifiedVars, configuredParsers, context) {
  if (fn.params.length > 1 || (fn.params.length === 1 && fn.params[0].type !== "Identifier")) {
    return null;
  }
  let expr =
    fn.body.type !== "BlockStatement"
      ? fn.body
      : fn.body.body.length === 1 && fn.body.body[0].type === "ReturnStatement"
        ? fn.body.body[0].argument
        : null;
  expr = unwrapExpression(expr);
  if (expr?.type === "AwaitExpression") expr = unwrapExpression(expr.argument);
  if (
    expr?.type === "CallExpression" &&
    isParserCallee(unwrapExpression(expr.callee), resolve, configuredParsers, context)
  ) {
    expr = unwrapExpression(expr.arguments[0]);
  }
  if (expr?.type !== "CallExpression") return null;
  const readInfo = isReadCall(expr, resolve, promisifiedVars, context);
  if (!readInfo || readInfo.isCallback) return null;
  return { paramName: fn.params[0]?.name ?? null, pathExpr: expr.arguments[0] };
}

function collectSources({
  calls,
  helperCandidates,
  resolve,
  resolvePath,
  promisifiedVars,
  parserBindings,
  context,
}) {
  const readHelpers = new Map();
  for (const { id, fn } of helperCandidates) {
    const info = readHelperInfo(fn, resolve, promisifiedVars, parserBindings, context);
    if (info) readHelpers.set(resolveVariable(id, context), info);
  }
  const sources = [];
  for (const call of calls) {
    const readInfo = isReadCall(call, resolve, promisifiedVars, context);
    if (readInfo) {
      const pathArg = call.arguments[0];
      const resolved = resolvePath(pathArg);
      if (resolved?.kind !== "repo") continue;
      let callbackParam = null;
      if (readInfo.isCallback) {
        const cb = unwrapExpression(call.arguments.at(-1));
        if (!isFunction(cb) || !cb.params[1]) continue;
        callbackParam = cb.params[1];
      }
      sources.push({
        node: call,
        stmt: sourceDeclarationLineNode(call),
        pathKey: resolved.key,
        pathVar: pathVariable(pathArg, context),
        callbackParam,
      });
      continue;
    }
    const callee = unwrapExpression(call.callee);
    const helper =
      callee?.type === "Identifier" ? readHelpers.get(resolveVariable(callee, context)) : null;
    if (!helper) continue;
    const env = helper.paramName ? new Map([[helper.paramName, call.arguments[0]]]) : null;
    const resolved = resolvePath(helper.pathExpr, env);
    if (resolved?.kind !== "repo") continue;
    sources.push({
      node: call,
      stmt: sourceDeclarationLineNode(call),
      pathKey: resolved.key,
      pathVar: helper.paramName ? pathVariable(call.arguments[0], context) : null,
      callbackParam: null,
    });
  }
  return sources;
}

module.exports = { collectSources, isParserCallee };
