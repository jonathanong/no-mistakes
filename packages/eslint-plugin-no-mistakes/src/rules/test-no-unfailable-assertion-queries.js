"use strict";

const { memberName, propertyName, unwrap } = require("./test-message-expect");
const { moduleMember } = require("./test-no-wall-time-assertions-bindings");
const { literalString } = require("./module-mock-helpers");
const { immutableConstDef } = require("./test-no-unfailable-assertion-assertions");

const THROWING_QUERY_PATTERN = /^(?:get|find)(?:All)?By[A-Z]/;

function testingLibraryExport(node, context) {
  const member = moduleMember(unwrap(node), context);
  return member && member.source.startsWith("@testing-library/") ? member.name : null;
}

function isQueryContainer(node, context) {
  let current = unwrap(node);
  if (current.type === "AwaitExpression") current = unwrap(current.argument);
  if (testingLibraryExport(current, context) === "screen") return true;
  if (current.type === "CallExpression") {
    const name = testingLibraryExport(current.callee, context);
    return name === "render" || name === "within";
  }
  if (current.type !== "Identifier") return false;
  const decl = immutableConstDef(current, context);
  return decl?.id.type === "Identifier" && isQueryContainer(decl.init, context);
}

function destructuredPropertyName(pattern, localName) {
  for (const prop of pattern.properties) {
    if (prop.type !== "Property") continue;
    let target = prop.value;
    while (target.type === "AssignmentPattern") target = target.left;
    if (target.type === "Identifier" && target.name === localName) {
      return prop.computed ? literalString(prop.key) : propertyName(prop.key);
    }
  }
  return null;
}

function throwingQueryCallee(callee, context) {
  const current = unwrap(callee);
  if (current.type === "MemberExpression") {
    const name = memberName(current);
    return name && THROWING_QUERY_PATTERN.test(name) && isQueryContainer(current.object, context)
      ? name
      : null;
  }
  if (current.type !== "Identifier") return null;
  const direct = testingLibraryExport(current, context);
  if (direct && THROWING_QUERY_PATTERN.test(direct)) return direct;
  const decl = immutableConstDef(current, context);
  if (!decl) return null;
  if (decl.id.type === "Identifier") return throwingQueryCallee(decl.init, context);
  if (decl.id.type === "ObjectPattern" && isQueryContainer(decl.init, context)) {
    const name = destructuredPropertyName(decl.id, current.name);
    return name && THROWING_QUERY_PATTERN.test(name) ? name : null;
  }
  return null;
}

function throwingQueryName(node, context) {
  let current = unwrap(node);
  if (current.type === "AwaitExpression") current = unwrap(current.argument);
  if (current.type === "CallExpression") return throwingQueryCallee(current.callee, context);
  if (current.type !== "Identifier") return null;
  const decl = immutableConstDef(current, context);
  return decl?.id.type === "Identifier" ? throwingQueryName(decl.init, context) : null;
}

module.exports = { throwingQueryName };
