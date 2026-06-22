"use strict";

const TEST_CALLEES = new Set(["it", "test", "describe"]);
const SETUP_CALLEES = new Set(["beforeEach", "afterEach", "beforeAll", "afterAll"]);
const PER_TEST_CALLEES = new Set(["beforeEach", "afterEach"]);
const TEST_MODIFIER_NAMES = new Set(
  "only skip skipIf runIf concurrent todo fail fails fixme parallel serial sequential".split(" "),
);
const TEST_TABLE_NAMES = new Set(["each", "for"]);

function calleeName(node) {
  if (node?.type === "Identifier") return node.name;
  if (node?.type === "MemberExpression" && !node.computed) return calleeName(node.object);
  if (node?.type === "CallExpression") return calleeName(node.callee);
  return null;
}

function propertyName(node) {
  return node.type === "Literal" ? String(node.value) : node.name;
}

function importSpecifierName(node) {
  const imported = node.imported;
  if (!imported) return null;
  return imported.type === "Literal" ? String(imported.value) : imported.name;
}

function isKnownTestCallee(callee, testCallees = TEST_CALLEES) {
  if (callee?.type === "CallExpression") return isKnownTestCallee(callee.callee, testCallees);
  if (callee?.type === "Identifier") return testCallees.has(callee.name);
  if (callee?.type !== "MemberExpression" || callee.computed) return false;
  const prop = propertyName(callee.property);
  if (!prop) return false;
  if (
    TEST_CALLEES.has(prop) &&
    callee.object?.type === "Identifier" &&
    testCallees.has(callee.object.name)
  ) {
    return true;
  }
  return (
    (TEST_MODIFIER_NAMES.has(prop) || TEST_TABLE_NAMES.has(prop)) &&
    isKnownTestCallee(callee.object, testCallees)
  );
}

function isTestExtendCall(node, testCallees = TEST_CALLEES) {
  if (node?.type !== "CallExpression") return false;
  const callee = node.callee;
  if (callee?.type !== "MemberExpression" || callee.computed) return false;
  return (
    propertyName(callee.property) === "extend" &&
    (isKnownTestCallee(callee.object, testCallees) || isTestExtendCall(callee.object, testCallees))
  );
}

function isTestCall(node, testCallees = TEST_CALLEES) {
  return isKnownTestCallee(node.callee, testCallees);
}

function setupCallbackKind(node, testCallees = TEST_CALLEES) {
  const name = calleeName(node.callee);
  if (PER_TEST_CALLEES.has(name)) return "per-test";
  if (name === "beforeAll") return "before-once";
  if (name === "afterAll") return "once";
  if (node.callee.type !== "MemberExpression" || !testCallees.has(name)) return null;
  const prop = propertyName(node.callee.property);
  if (PER_TEST_CALLEES.has(prop)) return "per-test";
  if (prop === "beforeAll") return "before-once";
  return prop === "afterAll" ? "once" : null;
}

module.exports = {
  SETUP_CALLEES,
  calleeName,
  importSpecifierName,
  isKnownTestCallee,
  isTestCall,
  isTestExtendCall,
  propertyName,
  setupCallbackKind,
};
