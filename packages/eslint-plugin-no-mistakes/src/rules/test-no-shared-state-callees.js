"use strict";

const { unwrapTransparentParent } = require("./async-ast");
const { logicalCall } = require("./logical-call");
const { literalString } = require("./module-mock-helpers");

const TEST_CALLEES = new Set(["it", "test", "describe"]);
const SETUP_CALLEES = new Set(["beforeEach", "afterEach", "beforeAll", "afterAll"]);
const PER_TEST_CALLEES = new Set(["beforeEach", "afterEach"]);
const TEST_MODIFIER_NAMES = new Set(
  "only skip skipIf runIf concurrent todo fail fails fixme parallel serial sequential".split(" "),
);
const TEST_TABLE_NAMES = new Set(["each", "for"]);

function staticPropertyName(node) {
  if (!node) return null;
  if (typeof node.name === "string" && node.type !== "Literal") return node.name;
  return literalString(node);
}

function calleeName(node) {
  while (node?.type === "ChainExpression") node = node.expression;
  if (node?.type === "Identifier") return node.name;
  if (node?.type === "MemberExpression") {
    if (node.computed && node.property?.type === "Identifier") return null;
    if (staticPropertyName(node.property)) return calleeName(node.object);
  }
  if (node?.type === "CallExpression") return calleeName(node.callee);
  return null;
}

function propertyName(node) {
  return staticPropertyName(node);
}

function importSpecifierName(node) {
  const imported = node.imported;
  if (!imported) return null;
  return imported.type === "Literal" ? String(imported.value) : imported.name;
}

function isKnownTestCallee(callee, testCallees = TEST_CALLEES, namespaces = new Set()) {
  while (callee?.type === "ChainExpression") callee = callee.expression;
  if (callee?.type === "SequenceExpression" && callee.expressions.length > 0) {
    return isKnownTestCallee(callee.expressions.at(-1), testCallees, namespaces);
  }
  if (
    callee?.type === "MemberExpression" &&
    callee.computed &&
    callee.property?.type === "Identifier"
  ) {
    return false;
  }
  if (callee?.type === "MemberExpression") {
    const forwarded = staticPropertyName(callee.property);
    if (!callee.computed && (forwarded === "call" || forwarded === "apply"))
      return isKnownTestCallee(callee.object, testCallees, namespaces);
  }
  if (callee?.type === "CallExpression")
    return isKnownTestCallee(callee.callee, testCallees, namespaces);
  if (callee?.type === "Identifier") return testCallees.has(callee.name);
  if (callee?.type !== "MemberExpression") return false;
  const prop = propertyName(callee.property);
  if (!prop) return false;
  if (
    TEST_CALLEES.has(prop) &&
    callee.object?.type === "Identifier" &&
    (testCallees.has(callee.object.name) || namespaces.has(callee.object.name))
  ) {
    return true;
  }
  return (
    (TEST_MODIFIER_NAMES.has(prop) || TEST_TABLE_NAMES.has(prop)) &&
    isKnownTestCallee(callee.object, testCallees, namespaces)
  );
}

function isTestExtendCall(node, testCallees = TEST_CALLEES, namespaces) {
  if (node?.type !== "CallExpression") return false;
  const callee = node.callee;
  if (callee?.type !== "MemberExpression") return false;
  return (
    propertyName(callee.property) === "extend" &&
    (isKnownTestCallee(callee.object, testCallees, namespaces) ||
      isTestExtendCall(callee.object, testCallees, namespaces))
  );
}

function isTestCall(node, testCallees = TEST_CALLEES, namespaces) {
  return isKnownTestCallee(node.callee, testCallees, namespaces);
}

function setupHook(callee) {
  while (callee?.type === "ChainExpression") callee = callee.expression;
  if (callee?.type === "SequenceExpression" && callee.expressions?.length) {
    callee = callee.expressions.at(-1);
  }
  if (callee?.type === "MemberExpression" && !callee.computed) {
    const forwarded = staticPropertyName(callee.property);
    if (forwarded === "call" || forwarded === "apply") return setupHook(callee.object);
  }
  return callee;
}

function setupCallbackKind(node, testCallees = TEST_CALLEES, namespaces = new Set()) {
  const callee = setupHook(node?.callee);
  const name = calleeName(callee);
  if (PER_TEST_CALLEES.has(name)) return "per-test";
  if (name === "beforeAll") return "before-once";
  if (name === "afterAll") return "once";
  if (callee?.type !== "MemberExpression") return null;
  if (callee.computed && callee.property?.type === "Identifier") return null;
  const object =
    callee.object?.type === "ChainExpression" ? callee.object.expression : callee.object;
  const objectName = object?.type === "Identifier" ? object.name : name;
  if (!testCallees.has(objectName) && !namespaces.has(objectName)) return null;
  const prop = propertyName(callee.property);
  if (PER_TEST_CALLEES.has(prop)) return "per-test";
  if (prop === "beforeAll") return "before-once";
  return prop === "afterAll" ? "once" : null;
}

// A callback in a literal .apply list is still registered by its containing call.
// Other array/container children are not inline test callbacks.
function callbackCall(node) {
  const callback = unwrapTransparentParent(node);
  const parent = callback.parent;
  const call = parent?.type === "ArrayExpression" ? unwrapTransparentParent(parent).parent : parent;
  const invoked = registrationCall(call);
  return invoked?.arguments.includes(callback) ? invoked : null;
}

function registrationCall(node) {
  // Track the effective receiver through nested intrinsic forwarding.
  const invoked = logicalCall(node, () => false);
  return invoked && { ...node, ...invoked };
}

module.exports = {
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
};
