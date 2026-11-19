"use strict";

const { memberPropertyName } = require("./async-target-bindings");

const WRAPPERS = new Set([
  "TSNonNullExpression",
  "TSAsExpression",
  "TSSatisfiesExpression",
  "TSTypeAssertion",
  "ChainExpression",
]);
const NAME_SLOTS = {
  MemberExpression: "property",
  Property: "key",
  MethodDefinition: "key",
  PropertyDefinition: "key",
  FunctionDeclaration: "id",
  FunctionExpression: "id",
  ClassDeclaration: "id",
  ClassExpression: "id",
  VariableDeclarator: "id",
  LabeledStatement: "label",
  BreakStatement: "label",
  ContinueStatement: "label",
};
const BINDING_PARENTS =
  /^(ArrayPattern|RestElement|Import.*Specifier|ExportSpecifier|CatchClause)$/;
const KEYS = ["reference", "declaration", "property", "construct"];

function compile(target) {
  const out = {};
  for (const kind of KEYS) {
    const pattern = target[`${kind}NamePattern`];
    out[kind] = pattern ? new RegExp(pattern) : null;
  }
  return out;
}

function invoked(node) {
  let current = node;
  while (WRAPPERS.has(current.parent.type)) current = current.parent;
  return current.parent.callee === current;
}

function isReference(node) {
  const parent = node.parent;
  if (BINDING_PARENTS.test(parent.type)) return false;
  if (parent[NAME_SLOTS[parent.type]] === node && !parent.computed) return false;
  if (parent.params?.includes(node)) return false;
  if (parent.type === "AssignmentPattern" && parent.left === node) return false;
  return !(parent.type === "Property" && parent.parent.type === "ObjectPattern");
}

function mergeVisitors(...visitors) {
  const merged = {};
  for (const visitor of visitors) {
    for (const [key, handler] of Object.entries(visitor)) {
      const previous = merged[key];
      merged[key] = previous
        ? (node) => {
            previous(node);
            handler(node);
          }
        : handler;
    }
  }
  return merged;
}

function strictVisitors(targets, report) {
  const compiled = targets.map((target) => [target, compile(target)]);
  function check(kind, node, name, accept = () => true) {
    if (name === null) return;
    for (const [target, patterns] of compiled) {
      if (patterns[kind]?.test(name) && accept(target)) report(node, target, name);
    }
  }
  function declaration(node) {
    if (node.id?.type === "Identifier") check("declaration", node.id, node.id.name);
  }
  function key(node) {
    check("property", node, memberPropertyName({ property: node.key, computed: node.computed }));
  }
  function reference(node, name) {
    if (!invoked(node)) check("reference", node, name);
  }
  return {
    Identifier(node) {
      if (isReference(node)) reference(node, node.name);
    },
    MemberExpression(node) {
      reference(node, memberPropertyName(node));
    },
    FunctionDeclaration: declaration,
    ClassDeclaration: declaration,
    VariableDeclarator: declaration,
    MethodDefinition: key,
    PropertyDefinition: key,
    Property(node) {
      if (node.parent.type === "ObjectExpression") key(node);
    },
    NewExpression(node) {
      let callee = node.callee;
      while (WRAPPERS.has(callee.type)) callee = callee.expression;
      const name =
        callee.type === "Identifier"
          ? callee.name
          : callee.type === "MemberExpression"
            ? memberPropertyName(callee)
            : null;
      const spread = node.arguments.some((argument) => argument.type === "SpreadElement");
      check("construct", node, name, (target) => {
        const count = target.constructArgumentCount;
        return count === undefined || (!spread && node.arguments.length === count);
      });
    },
  };
}

module.exports = { mergeVisitors, strictVisitors };
