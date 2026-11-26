"use strict";

const { repoRelativeFilename } = require("./module-mock-helpers");
const postgresExecutor = require("./postgres-executor");
const postgresQueryText = require("./postgres-query-text");

const { firstCallArgument, isDatabaseCall, memberPropertyName } = postgresExecutor;
const { childNodes, quasiText, resolveVariable, unwrapTs } = postgresQueryText;

const SCREAMING_CASE = /^[A-Z][A-Z0-9_]*$/;

function isOwnerFile(filename, owners) {
  if (!filename || !owners?.length) return false;
  const normalized = String(filename).replace(/\\/g, "/");
  const relative = repoRelativeFilename(filename);
  return owners.some(
    (owner) => pathMatchesOwner(normalized, owner) || pathMatchesOwner(relative, owner),
  );
}

function pathMatchesOwner(path, owner) {
  const needle = String(owner ?? "").replace(/\\/g, "/");
  if (!needle) return false;
  if (path === needle) return true;
  const suffix = needle.replace(/^\/+/, "");
  return Boolean(suffix) && path.endsWith(`/${suffix}`);
}

function callName(node) {
  const callee = unwrapTs(node?.callee);
  if (!callee) return null;
  if (callee.type === "Identifier") return callee.name;
  if (callee.type === "MemberExpression") return memberPropertyName(callee);
  return null;
}

function isChunkCall(node, chunkFunctionNames) {
  const name = callName(node);
  return Boolean(name && chunkFunctionNames.includes(name));
}

function forOfRightHandSide(defNode) {
  const declaration = defNode?.parent;
  if (declaration?.type !== "VariableDeclaration") return undefined;
  const loop = declaration.parent;
  if (loop?.type !== "ForOfStatement" || loop.left !== declaration) return undefined;
  return loop.right;
}

function isReassignedBinding(variable) {
  return (variable?.references ?? []).some((reference) => reference.isWrite() && !reference.init);
}

function isStaticallyBounded(source, chunkFunctionNames, context, seen = new Set()) {
  source = unwrapTs(source);
  if (!source) return false;
  if (source.type === "ArrayExpression") {
    return (source.elements ?? []).every(
      (element) =>
        !element ||
        element.type !== "SpreadElement" ||
        isStaticallyBounded(element.argument, chunkFunctionNames, context, seen),
    );
  }
  if (source.type === "Identifier" && SCREAMING_CASE.test(source.name)) return true;
  if (source.type === "CallExpression" && isChunkCall(source, chunkFunctionNames)) return true;
  if (source.type === "Identifier" && context) {
    const variable = resolveVariable(source, context);
    if (!variable || seen.has(variable)) return false;
    const def = variable.defs?.find(
      (item) => item.type === "Variable" && item.node?.id?.type === "Identifier",
    );
    const init = def?.node?.init;
    if (init && init !== source) {
      if (isReassignedBinding(variable)) return false;
      const nextSeen = new Set(seen);
      nextSeen.add(variable);
      return isStaticallyBounded(init, chunkFunctionNames, context, nextSeen);
    }
    const iterated = forOfRightHandSide(def?.node);
    if (iterated) {
      if (isReassignedBinding(variable)) return false;
      const right = unwrapTs(iterated);
      return Boolean(right?.type === "CallExpression" && isChunkCall(right, chunkFunctionNames));
    }
  }
  return false;
}

function staticPromiseMember(node) {
  if (!node.computed) return node.property?.name ?? null;
  const key = unwrapTs(node.property);
  if (key?.type === "Literal" && typeof key.value === "string") return key.value;
  if (key?.type === "TemplateLiteral" && key.expressions.length === 0) {
    return quasiText(key.quasis[0]);
  }
  return null;
}

function isPromiseAllCallee(node) {
  const callee = unwrapTs(node);
  if (callee?.type !== "MemberExpression") return false;
  if (staticPromiseMember(callee) !== "all") return false;
  const object = unwrapTs(callee.object);
  return object?.type === "Identifier" && object.name === "Promise";
}

function mapCallArgument(node) {
  const argument = unwrapTs(firstCallArgument(node));
  if (argument?.type !== "CallExpression") return null;
  const callee = unwrapTs(argument.callee);
  if (callee?.type !== "MemberExpression") return null;
  if (memberPropertyName(callee) !== "map") return null;
  return {
    source: unwrapTs(callee.object),
    callback: argument.arguments[0],
    mapCall: argument,
  };
}

function containsDatabaseCall(node, bindings) {
  if (!node || typeof node !== "object") return false;
  if (isDatabaseCall(node, bindings)) return true;
  return childNodes(node).some((child) => containsDatabaseCall(child, bindings));
}

function functionFromDefinition(def) {
  const node = def?.node;
  if (!node) return null;
  if (
    node.type === "FunctionDeclaration" ||
    node.type === "FunctionExpression" ||
    node.type === "ArrowFunctionExpression"
  ) {
    return node;
  }
  const init = node.init;
  if (
    init &&
    (init.type === "FunctionExpression" ||
      init.type === "ArrowFunctionExpression" ||
      init.type === "FunctionDeclaration")
  ) {
    return init;
  }
  return null;
}

function callbackContainsExecutor(callback, bindings, context) {
  callback = unwrapTs(callback);
  if (!callback) return false;
  if (
    callback.type === "ArrowFunctionExpression" ||
    callback.type === "FunctionExpression" ||
    callback.type === "FunctionDeclaration"
  ) {
    return containsDatabaseCall(callback, bindings);
  }
  if (bindings?.resolveCall && bindings.resolveCall.reference(callback)) return true;
  if (callback.type !== "Identifier") return false;
  if (!bindings?.resolveCall && bindings?.has(callback.name)) return true;
  if (!context) return false;
  const variable = resolveVariable(callback, context);
  if (
    bindings?.resolveCall &&
    (isReassignedBinding(variable) ||
      variable?.defs.some((def) => def.type === "Variable" && def.parent.kind !== "const"))
  )
    return false;
  for (const def of variable?.defs ?? []) {
    const fn = functionFromDefinition(def);
    if (fn && containsDatabaseCall(fn, bindings)) return true;
  }
  return false;
}

module.exports = {
  ...postgresExecutor,
  ...postgresQueryText,
  callbackContainsExecutor,
  containsDatabaseCall,
  isOwnerFile,
  isPromiseAllCallee,
  isStaticallyBounded,
  mapCallArgument,
};
