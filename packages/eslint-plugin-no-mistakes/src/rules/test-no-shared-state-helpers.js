"use strict";

const TEST_CALLEES = new Set(["it", "test", "describe"]);
const SETUP_CALLEES = new Set(["beforeEach", "afterEach", "beforeAll", "afterAll"]);
const MUTATING_METHODS = new Set(
  "add clear copyWithin delete fill pop push reverse set shift sort splice unshift".split(" "),
);
const MUTABLE_CONSTRUCTORS = new Set(["Map", "Set", "WeakMap", "WeakSet"]);
const FUNCTION_NODES = new Set([
  "FunctionDeclaration",
  "FunctionExpression",
  "ArrowFunctionExpression",
]);

function calleeName(node) {
  if (node?.type === "Identifier") return node.name;
  if (node?.type === "MemberExpression" && !node.computed) return calleeName(node.object);
  if (node?.type === "CallExpression") return calleeName(node.callee);
  return null;
}

function isTestCall(node) {
  return TEST_CALLEES.has(calleeName(node.callee));
}

function setupCallbackKind(node) {
  const name = calleeName(node.callee);
  return name === "beforeEach" || name === "afterEach"
    ? "per-test"
    : name === "beforeAll" || name === "afterAll"
      ? "once"
      : null;
}

function isSetupCall(node) {
  return SETUP_CALLEES.has(calleeName(node.callee));
}

function collectPatternNames(node, names = new Set()) {
  if (!node) return names;
  if (node.type === "Identifier") {
    names.add(node.name);
    return names;
  }
  const children =
    node.type === "ObjectPattern"
      ? node.properties.map((property) => property.value || property.argument)
      : node.type === "ArrayPattern"
        ? node.elements
        : node.type === "RestElement"
          ? [node.argument]
          : node.type === "AssignmentPattern"
            ? [node.left]
            : [];
  for (const child of children) collectPatternNames(child, names);
  return names;
}

function mutationRootName(node) {
  if (node.type === "Identifier") return node.name;
  if (node.type === "MemberExpression") return mutationRootName(node.object);
  return null;
}

function propertyName(node) {
  return node.type === "Literal" ? String(node.value) : node.name;
}

function mutatingCallRootName(node) {
  return node.callee.type === "MemberExpression" &&
    MUTATING_METHODS.has(propertyName(node.callee.property))
    ? mutationRootName(node.callee.object)
    : null;
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
  return node.parent?.type === "CallExpression" && isSetupCall(node.parent);
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
  return Object.entries(node)
    .filter(([key]) => !["parent", "loc", "range", "tokens", "comments"].includes(key))
    .flatMap(([, value]) => (Array.isArray(value) ? value : [value]))
    .filter((value) => value && typeof value.type === "string");
}

function namedCallbackArgument(args) {
  for (let index = args.length - 1; index >= 0; index -= 1) {
    if (args[index].type === "Identifier") return args[index];
  }
}

function firstNamedCallbackArgument(args) {
  return args[0]?.type === "Identifier" ? args[0] : undefined;
}

function createCleanupTracker() {
  const mutablesBySuite = new Map();
  const suiteStack = [];
  let activeSuiteKey;
  let replaySuiteKey;
  let nextSuiteId = 0;

  function currentSuiteKey() {
    return replaySuiteKey ?? suiteStack.join("/");
  }

  function has(name, suiteKey) {
    for (const [cleanupSuiteKey, names] of mutablesBySuite) {
      if (!names.has(name)) continue;
      if (!cleanupSuiteKey || suiteKey === cleanupSuiteKey) return true;
      if (suiteKey.startsWith(`${cleanupSuiteKey}/`)) return true;
    }
    return false;
  }

  return {
    beginSetup(kind, suiteKey = currentSuiteKey()) {
      activeSuiteKey = kind === "per-test" ? suiteKey : undefined;
    },
    clearReplaySuite() {
      replaySuiteKey = undefined;
    },
    currentSuiteKey,
    endSetup() {
      activeSuiteKey = undefined;
    },
    enterSuite() {
      suiteStack.push(String(nextSuiteId++));
    },
    exitSuite() {
      suiteStack.pop();
    },
    has,
    remember(name) {
      if (!name || activeSuiteKey === undefined) return;
      const names = mutablesBySuite.get(activeSuiteKey) ?? new Set();
      names.add(name);
      mutablesBySuite.set(activeSuiteKey, names);
    },
    setReplaySuite(suiteKey) {
      replaySuiteKey = suiteKey;
    },
  };
}

module.exports = {
  childNodes,
  collectPatternNames,
  createCleanupTracker,
  firstNamedCallbackArgument,
  isCalledFunction,
  isFunctionNode,
  isInlineSetupCallback,
  isInlineTestCallback,
  isMutableInitializer,
  calleeName,
  isSetupCall,
  isTestCall,
  mutatingCallPropertyName,
  mutatingCallRootName,
  mutationRootName,
  namedCallbackArgument,
  setupCallbackKind,
};
