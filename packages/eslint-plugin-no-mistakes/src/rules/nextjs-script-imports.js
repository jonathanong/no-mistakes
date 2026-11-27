"use strict";

const { importFact } = require("./import-resolution");
const { literalString } = require("./logical-call");

const SKIP_KEYS = new Set(["comments", "loc", "parent", "range", "tokens"]);

function noteModule(moduleName, state) {
  if (moduleName == null) state.unresolved = true;
  else if (moduleName.startsWith("next/")) state.next = true;
}

function noteFact(fact, state) {
  if (!fact) return;
  if (fact.unresolved) state.unresolved = true;
  else if (!fact.typeOnly && typeof fact.module === "string") noteModule(fact.module, state);
}

function externalModule(node) {
  const reference = node.moduleReference;
  if (reference?.type !== "TSExternalModuleReference") return undefined;
  return literalString(reference.expression);
}

function typeOnlyImportNode(node) {
  if (node.importKind === "type") return true;
  const specifiers = node.specifiers;
  return (
    Array.isArray(specifiers) &&
    specifiers.length > 0 &&
    specifiers.every((specifier) => specifier.importKind === "type")
  );
}

function visit(node, context, state, seen) {
  if (!node || typeof node !== "object" || seen.has(node)) return;
  seen.add(node);
  if (node.type === "ImportDeclaration" && typeof node.source?.value === "string") {
    if (!typeOnlyImportNode(node)) noteModule(node.source.value, state);
  } else if (node.type === "ImportExpression") {
    noteModule(node.source ? literalString(node.source) : null, state);
  } else if (node.type === "TSImportEqualsDeclaration") {
    if (node.importKind !== "type") {
      const moduleName = externalModule(node);
      if (moduleName !== undefined) noteModule(moduleName, state);
      else noteFact(node.id ? importFact(node.id, context) : null, state);
    }
  } else if (node.type === "CallExpression") {
    noteFact(importFact(node, context), state);
  }
  for (const key of Object.keys(node)) {
    if (SKIP_KEYS.has(key)) continue;
    const value = node[key];
    if (Array.isArray(value)) {
      for (const child of value) {
        if (child && typeof child.type === "string") visit(child, context, state, seen);
      }
    } else if (value && typeof value.type === "string") {
      visit(value, context, state, seen);
    }
  }
}

function nextImportState(program, context) {
  const state = { next: false, unresolved: false };
  visit(program, context, state, new Set());
  return state;
}

module.exports = { nextImportState };
