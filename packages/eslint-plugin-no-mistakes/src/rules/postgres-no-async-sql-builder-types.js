"use strict";

const { propertyName } = require("./module-mock-helpers");

// `any`, `unknown`, and `object` are omitted. Each accepts a Promise, so a
// parameter or return of those types is unknown rather than a non-Promise.
// A type literal is handled separately: `{}` accepts a Promise the same way
// `object` does. Only a required member a Promise does not already have
// (not `then` / `catch` / `finally`, and not an `Object.prototype` member)
// makes the literal a non-Promise.
const NON_PROMISE_TYPES = new Set([
  "TSArrayType",
  "TSBigIntKeyword",
  "TSBooleanKeyword",
  "TSConstructorType",
  "TSFunctionType",
  "TSIntrinsicKeyword",
  "TSLiteralType",
  "TSNeverKeyword",
  "TSNullKeyword",
  "TSNumberKeyword",
  "TSStringKeyword",
  "TSSymbolKeyword",
  "TSThisType",
  "TSTupleType",
  "TSUndefinedKeyword",
  "TSVoidKeyword",
]);

const PROMISE_MEMBERS = new Set([
  "then",
  "catch",
  "finally",
  "constructor",
  "toString",
  "toLocaleString",
  "valueOf",
  "hasOwnProperty",
  "isPrototypeOf",
  "propertyIsEnumerable",
]);

// Object.prototype names, plus the members a function value already has.
const FUNCTION_MEMBERS = new Set([
  "constructor",
  "toString",
  "toLocaleString",
  "valueOf",
  "hasOwnProperty",
  "isPrototypeOf",
  "propertyIsEnumerable",
  "name",
  "length",
  "call",
  "apply",
  "bind",
  "prototype",
  "arguments",
  "caller",
]);

const UNRESOLVED_TYPES = new Set([
  "TSConditionalType",
  "TSImportType",
  "TSIndexedAccessType",
  "TSInferType",
  "TSMappedType",
  "TSNamedTupleMember",
  "TSTemplateLiteralType",
  "TSTypeQuery",
]);

function combine(results) {
  let unknown = false;
  for (const result of results) {
    if (result === "requires") return "requires";
    if (result === "unknown") unknown = true;
  }
  return unknown ? "unknown" : "allows";
}

function walkAst(context, node, visit) {
  visit(node);
  for (const key of context.sourceCode.visitorKeys[node.type]) {
    const value = node[key];
    if (Array.isArray(value)) {
      for (const child of value) walkAst(context, child, visit);
    } else if (value?.type) walkAst(context, value, visit);
  }
}

function literalName(node) {
  return node?.type === "Literal" ? String(node.value) : null;
}

function staticKeyName(node) {
  if (node.type === "MemberExpression")
    return node.computed ? literalName(node.property) : propertyName(node.property);
  if (node.computed) return literalName(node.key);
  return propertyName(node.key);
}

function remember(map, name, value) {
  if (!name) return;
  if (map.has(name)) map.set(name, { ambiguous: true });
  else map.set(name, { ambiguous: false, value });
}

function collectTypes(context, program) {
  const aliases = new Map();
  const interfaces = new Map();
  const classes = new Map();
  const classList = [];
  walkAst(context, program, (node) => {
    if (node.type === "TSTypeAliasDeclaration")
      remember(aliases, node.id?.name, node.typeAnnotation);
    else if (node.type === "TSInterfaceDeclaration") remember(interfaces, node.id?.name, node);
    else if (node.type === "ClassDeclaration" || node.type === "ClassExpression") {
      classList.push(node);
      if (node.type === "ClassDeclaration") remember(classes, node.id?.name, node);
    }
  });
  return { aliases, interfaces, classes, classList };
}

function lookup(map, name) {
  const found = name ? map.get(name) : null;
  return found && !found.ambiguous ? found.value : null;
}

function referenceName(typeName) {
  if (typeName.type === "Identifier") return typeName.name;
  // TSTypeReference.typeName is an identifier or a qualified name whose right
  // half is an identifier. Only globalThis.Promise is the Promise type.
  const left = referenceName(typeName.left);
  const right = typeName.right.name;
  return left === "globalThis" && right === "Promise" ? "Promise" : null;
}

function unwrapType(node) {
  let current = node;
  while (
    current &&
    (current.type === "TSTypeAnnotation" ||
      current.type === "TSParenthesizedType" ||
      current.type === "TSTypeOperator")
  ) {
    current = current.typeAnnotation;
  }
  return current;
}

function typeLiteralFlag(type, inherited) {
  // "no" only when a required member is outside `inherited`. Optional members
  // and an empty literal stay unknown.
  for (const member of type.members) {
    if (
      member.optional ||
      (member.type !== "TSPropertySignature" && member.type !== "TSMethodSignature")
    ) {
      continue;
    }
    const name = staticKeyName(member);
    if (name && !inherited.has(name)) return "no";
  }
  return "unknown";
}

function typeLiteralPromiseFlag(type) {
  return typeLiteralFlag(type, PROMISE_MEMBERS);
}

function typePromiseFlag(typeNode, types, seen) {
  const type = unwrapType(typeNode);
  if (type.type === "TSTypeLiteral") return typeLiteralPromiseFlag(type);
  if (NON_PROMISE_TYPES.has(type.type)) return "no";
  if (UNRESOLVED_TYPES.has(type.type)) return "unknown";
  if (type.type === "TSUnionType" || type.type === "TSIntersectionType") {
    let unknown = false;
    for (const part of type.types) {
      const flag = typePromiseFlag(part, types, seen);
      if (flag === "yes") return "yes";
      if (flag === "unknown") unknown = true;
    }
    return unknown ? "unknown" : "no";
  }
  if (type.type !== "TSTypeReference") return "unknown";
  const name = referenceName(type.typeName);
  if (name === "Promise") return "yes";
  if (!name) return "unknown";
  const alias = lookup(types.aliases, name);
  if (!alias) return "unknown";
  if (seen.has(name)) return "unknown";
  seen.add(name);
  return typePromiseFlag(alias, types, seen);
}

function annotationContract(typeNode, types) {
  if (!typeNode) return "allows";
  const flag = typePromiseFlag(typeNode, types, new Set());
  if (flag === "yes") return "requires";
  if (flag === "unknown") return "unknown";
  return "allows";
}

module.exports = {
  FUNCTION_MEMBERS,
  NON_PROMISE_TYPES,
  annotationContract,
  collectTypes,
  combine,
  lookup,
  referenceName,
  staticKeyName,
  typeLiteralFlag,
  unwrapType,
  walkAst,
};
