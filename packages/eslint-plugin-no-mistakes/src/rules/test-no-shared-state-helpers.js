"use strict";

const { collectPatternNames } = require("./ast-pattern-names");
const { createCleanupTracker } = require("./test-no-shared-state-cleanup");
const {
  SETUP_CALLEES,
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
  if (node.type === "Identifier") return node.name;
  if (node.type === "MemberExpression") return mutationRootName(node.object);
  return null;
}

function mutationPath(node) {
  if (node.type === "Identifier") return node.name;
  if (node.type !== "MemberExpression") return null;
  const objectPath = mutationPath(node.object);
  const property =
    node.computed && node.property.type !== "Literal" ? null : propertyName(node.property);
  return objectPath && property ? `${objectPath}.${property}` : null;
}

function mutatingCallTarget(node) {
  if (
    node.callee.type !== "MemberExpression" ||
    !MUTATING_METHODS.has(propertyName(node.callee.property))
  ) {
    return { name: null, path: null };
  }
  return {
    name: mutationRootName(node.callee.object),
    path: mutationPath(node.callee.object),
  };
}

function mutatingCallPropertyName(node) {
  return node.callee.type === "MemberExpression" ? propertyName(node.callee.property) : null;
}

function isFunctionNode(node) {
  return FUNCTION_NODES.has(node?.type);
}

function isInlineTestCallback(node) {
  return node.parent?.type === "CallExpression" && isTestCall(node.parent);
}

function isInlineSetupCallback(node) {
  const p = node.parent;
  return p?.type === "CallExpression" && SETUP_CALLEES.has(calleeName(p.callee));
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
  if (node.type === "CallExpression" && node.callee.type === "Identifier")
    return node.callee.name === name;
  if (isFunctionNode(node)) return false;
  return childNodes(node).some((child) => containsIdentifierCall(child, name));
}

function isMutableInitializer(node) {
  return Boolean(
    node &&
    (node.type === "ArrayExpression" ||
      node.type === "ObjectExpression" ||
      (node.type === "NewExpression" &&
        node.callee.type === "Identifier" &&
        MUTABLE_CONSTRUCTORS.has(node.callee.name))),
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
    if (args[index].type === "Identifier") return args[index];
  }
}

function firstNamedCallbackArgument(args) {
  return args[0]?.type === "Identifier" ? args[0] : undefined;
}

module.exports = {
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
