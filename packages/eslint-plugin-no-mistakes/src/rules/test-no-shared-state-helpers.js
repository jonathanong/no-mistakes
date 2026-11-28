"use strict";

const { unwrapExpression } = require("./async-ast");
const { collectPatternNames } = require("./ast-pattern-names");
const { createCleanupTracker } = require("./test-no-shared-state-cleanup");
const {
  SETUP_CALLEES,
  callbackCall,
  registrationCall,
  calleeName,
  importSpecifierName,
  isKnownTestCallee,
  isTestCall,
  isTestExtendCall,
  propertyName,
  setupCallbackKind,
} = require("./test-no-shared-state-callees");
const MUTATING_METHODS = new Set(
  "add clear copyWithin delete fill pop push reverse set shift sort splice unshift".split(" "),
);
const MUTABLE_CONSTRUCTORS = new Set(["Map", "Set", "WeakMap", "WeakSet"]);
const FUNCTION_NODES = new Set([
  "FunctionDeclaration",
  "FunctionExpression",
  "ArrowFunctionExpression",
]);

function mutationRootName(node) {
  const current = unwrapExpression(node);
  if (current.type === "Identifier") return current.name;
  if (current.type === "MemberExpression") return mutationRootName(current.object);
  return null;
}

function mutationPath(node) {
  const current = unwrapExpression(node);
  if (current.type === "Identifier") return current.name;
  if (current.type !== "MemberExpression") return null;
  const objectPath = mutationPath(current.object);
  const property =
    current.computed && current.property.type !== "Literal" ? null : propertyName(current.property);
  return objectPath && property ? `${objectPath}.${property}` : null;
}

function mutatingCallTarget(node) {
  const callee = unwrapExpression(node.callee);
  if (callee.type !== "MemberExpression" || !MUTATING_METHODS.has(propertyName(callee.property))) {
    return { name: null, path: null };
  }
  const object = unwrapExpression(callee.object);
  return {
    name: mutationRootName(object),
    path: mutationPath(object),
  };
}

function mutatingCallPropertyName(node) {
  const callee = unwrapExpression(node.callee);
  return callee.type === "MemberExpression" ? propertyName(callee.property) : null;
}

function isFunctionNode(node) {
  return FUNCTION_NODES.has(node?.type);
}

function isInlineTestCallback(node) {
  const call = callbackCall(node);
  return Boolean(call && isTestCall(call));
}

function isInlineSetupCallback(node) {
  const call = callbackCall(node);
  return Boolean(call && SETUP_CALLEES.has(calleeName(call.callee)));
}

function isCalledFunction(node) {
  if (node.parent?.type === "CallExpression" && node.parent.callee === node) return true;
  const declarator = node.parent?.type === "VariableDeclarator" ? node.parent : null;
  const name =
    node.type === "FunctionDeclaration"
      ? node.id?.name
      : declarator?.id?.type === "Identifier"
        ? declarator.id.name
        : null;
  const container = node.type === "FunctionDeclaration" ? node.parent : node.parent?.parent?.parent;
  return Boolean(name && container && containsIdentifierCall(container, name));
}

function containsIdentifierCall(node, name) {
  const current = unwrapExpression(node);
  if (current.type === "CallExpression") {
    const callee = unwrapExpression(current.callee);
    if (callee.type === "Identifier" && callee.name === name) return true;
  }
  if (isFunctionNode(current)) return false;
  return childNodes(current).some((child) => containsIdentifierCall(child, name));
}

function isMutableInitializer(node) {
  node = unwrapExpression(node);
  const callee = node?.type === "NewExpression" ? unwrapExpression(node.callee) : null;
  return Boolean(
    node &&
    (node.type === "ArrayExpression" ||
      node.type === "ObjectExpression" ||
      (callee?.type === "Identifier" && MUTABLE_CONSTRUCTORS.has(callee.name))),
  );
}

function childNodes(node) {
  const children = [];
  for (const key in node) {
    if (!Object.prototype.hasOwnProperty.call(node, key)) continue;
    if (
      key === "parent" ||
      key === "loc" ||
      key === "range" ||
      key === "tokens" ||
      key === "comments"
    ) {
      continue;
    }

    const value = node[key];
    if (Array.isArray(value)) {
      for (let i = 0; i < value.length; i += 1) {
        const child = value[i];
        if (child && typeof child.type === "string") {
          children.push(child);
        }
      }
    } else if (value && typeof value.type === "string") {
      children.push(value);
    }
  }
  return children;
}

function namedCallbackArgument(args) {
  for (let index = args.length - 1; index >= 0; index -= 1) {
    const argument = unwrapExpression(args[index]);
    if (argument.type === "Identifier") return argument;
  }
}

function firstNamedCallbackArgument(args) {
  const argument = args[0] && unwrapExpression(args[0]);
  return argument?.type === "Identifier" ? argument : undefined;
}

module.exports = {
  callbackCall,
  registrationCall,
  childNodes,
  collectPatternNames,
  createCleanupTracker,
  firstNamedCallbackArgument,
  importSpecifierName,
  isCalledFunction,
  isFunctionNode,
  isInlineSetupCallback,
  isInlineTestCallback,
  isMutableInitializer,
  isTestExtendCall,
  calleeName,
  isTestCall,
  isKnownTestCallee,
  mutatingCallPropertyName,
  mutatingCallTarget,
  mutationPath,
  mutationRootName,
  namedCallbackArgument,
  setupCallbackKind,
};
