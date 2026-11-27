"use strict";

const { memberPropertyName } = require("./async-target-bindings");

const WRAPPERS = new Set([
  "TSNonNullExpression",
  "TSAsExpression",
  "TSSatisfiesExpression",
  "TSTypeAssertion",
  "TSInstantiationExpression",
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
  TSInterfaceDeclaration: "id",
  TSTypeAliasDeclaration: "id",
  TSEnumDeclaration: "id",
  TSModuleDeclaration: "id",
  TSPropertySignature: "key",
  TSMethodSignature: "key",
  TSTypeParameter: "name",
  TSMappedType: "key",
  TSNamedTupleMember: "label",
};
const BINDING_PARENTS =
  /^(ArrayPattern|RestElement|Import.*Specifier|ExportSpecifier|CatchClause|TSDeclareFunction)$/;
const KEYS = ["reference", "declaration", "property", "destructure", "construct"];

function compile(target) {
  const out = {};
  for (const kind of KEYS) {
    const pattern = target[`${kind}NamePattern`];
    out[kind] = pattern ? new RegExp(pattern) : null;
  }
  return out;
}

function findingMessage(target, name, rewrite) {
  return (target.message || `Restricted API '${name}'. Use an allowed API instead.`) + rewrite;
}

function rememberFinding(seen, node, message) {
  let messages = seen.get(node);
  if (!messages) {
    messages = new Set();
    seen.set(node, messages);
  }
  if (messages.has(message)) return false;
  messages.add(message);
  return true;
}

function invoked(node) {
  let current = node;
  while (WRAPPERS.has(current.parent.type)) current = current.parent;
  return current.parent.callee === current;
}

function assignmentTarget(node) {
  const parent = node.parent;
  if (
    WRAPPERS.has(parent.type) ||
    parent.type === "ArrayPattern" ||
    parent.type === "RestElement" ||
    parent.type === "ObjectPattern"
  ) {
    return parent;
  }
  if (parent.type === "AssignmentPattern" && parent.left === node) return parent;
  if (
    parent.type === "Property" &&
    parent.value === node &&
    parent.parent.type === "ObjectPattern"
  ) {
    return parent;
  }
  return null;
}

function isWriteTarget(node) {
  let current = node;
  let enclosing = assignmentTarget(current);
  while (enclosing) {
    current = enclosing;
    enclosing = assignmentTarget(current);
  }
  const parent = current.parent;
  if (parent.type === "VariableDeclarator") return parent.id === current;
  if (parent.type === "AssignmentExpression") return parent.left === current;
  if (parent.type === "ForInStatement") return parent.left === current;
  if (parent.type === "ForOfStatement") return parent.left === current;
  return false;
}

function isTypePosition(node) {
  if (node.parent.type === "TSTypePredicate") return true;
  let current = node;
  while (current.parent?.type === "TSQualifiedName") current = current.parent;
  const parent = current.parent;
  if (parent?.type === "TSModuleDeclaration" && parent.id === current) return true;
  return (
    parent.type === "TSTypeReference" ||
    parent.type === "TSInterfaceHeritage" ||
    parent.type === "TSClassImplements" ||
    parent.type === "TSImportType"
  );
}

function isReference(node) {
  const parent = node.parent;
  const statement = parent.parent;
  if (
    parent.type === "ExportSpecifier" &&
    parent.local === node &&
    statement?.type === "ExportNamedDeclaration" &&
    !statement.source &&
    statement.exportKind !== "type" &&
    parent.exportKind !== "type"
  )
    return true;
  if (BINDING_PARENTS.test(parent.type)) return false;
  if (parent.type === "TSEnumMember" && parent.id === node) return false;
  if (parent.type === "TSIndexSignature" && parent.parameters.includes(node)) return false;
  if (parent[NAME_SLOTS[parent.type]] === node && !parent.computed) return false;
  if (parent.params?.includes(node)) return false;
  if (parent.type === "AssignmentPattern" && parent.left === node) return false;
  if (isTypePosition(node)) return false;
  if (parent.type === "Property" && parent.parent.type === "ObjectPattern") {
    return parent.computed && parent.key === node;
  }
  return true;
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
    if (isWriteTarget(node) || isTypePosition(node) || invoked(node)) return;
    check("reference", node, name);
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
      const kind = node.parent.type === "ObjectPattern" ? "destructure" : "property";
      check(kind, node, memberPropertyName({ property: node.key, computed: node.computed }));
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

module.exports = { findingMessage, mergeVisitors, rememberFinding, strictVisitors };
