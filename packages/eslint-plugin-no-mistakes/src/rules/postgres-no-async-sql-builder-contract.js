"use strict";

const {
  findContainingFunction,
  isFunction,
  isTransparentExpression,
  unwrapExpression,
} = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { propertyName } = require("./module-mock-helpers");

const NON_PROMISE_TYPES = new Set([
  "TSAnyKeyword",
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
  "TSObjectKeyword",
  "TSStringKeyword",
  "TSSymbolKeyword",
  "TSThisType",
  "TSTupleType",
  "TSTypeLiteral",
  "TSUndefinedKeyword",
  "TSUnknownKeyword",
  "TSVoidKeyword",
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

const PROMISE_USES = new Set(["then", "catch", "finally"]);

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

function typePromiseFlag(typeNode, types, seen) {
  const type = unwrapType(typeNode);
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

function callableReturnContract(typeNode, types, seen = new Set()) {
  const type = unwrapType(typeNode);
  if (!type) return "unknown";
  if (type.type === "TSUnionType") {
    // Any Promise-returning alternative can be what callers invoke.
    return combine(type.types.map((part) => callableReturnContract(part, types, seen)));
  }
  if (type.type === "TSIntersectionType")
    return combine(type.types.map((part) => callableReturnContract(part, types, seen)));
  if (type.type === "TSFunctionType") return annotationContract(type.returnType, types);
  if (type.type === "TSTypeLiteral") {
    const signature = type.members.find((member) => member.type === "TSCallSignatureDeclaration");
    return signature ? annotationContract(signature.returnType, types) : "unknown";
  }
  if (type.type === "TSTypeReference") {
    const name = referenceName(type.typeName);
    if (!name || name === "Promise") return "unknown";
    const alias = lookup(types.aliases, name);
    if (!alias) return "unknown";
    if (seen.has(name)) return "unknown";
    seen.add(name);
    return callableReturnContract(alias, types, seen);
  }
  if (NON_PROMISE_TYPES.has(type.type)) return "allows";
  return "unknown";
}

function valueContract(typeNode, types) {
  return combine([annotationContract(typeNode, types), callableReturnContract(typeNode, types)]);
}

function membersContract(members, name, types) {
  const member = members.find((item) => staticKeyName(item) === name);
  if (!member) return "missing";
  if (member.type === "TSMethodSignature") return annotationContract(member.returnType, types);
  // A named type element that is not a method is a property. Other type
  // elements have no static name, so they cannot be selected above.
  return valueContract(member.typeAnnotation, types);
}

function interfacePropertyContract(face, name, types, seen) {
  if (seen.has(face)) return "unknown";
  seen.add(face);
  const own = membersContract(face.body.body, name, types);
  if (own === "requires" || own === "unknown") return own;
  let result = own === "allows" ? "allows" : "missing";
  for (const extend of face.extends || []) {
    const parent = lookup(types.interfaces, extend.expression?.name);
    if (!parent) return "unknown";
    const inherited = interfacePropertyContract(parent, name, types, seen);
    if (inherited === "requires" || inherited === "unknown") return inherited;
    if (inherited === "allows") result = "allows";
  }
  return result;
}

function typePropertyContract(typeNode, name, types, seen) {
  const type = unwrapType(typeNode);
  if (type.type === "TSTypeLiteral") return membersContract(type.members, name, types);
  if (type.type === "TSUnionType") {
    const parts = type.types.map((part) => typePropertyContract(part, name, types, seen));
    if (parts.some((part) => part === "unknown" || part === "missing")) return "unknown";
    return parts.every((part) => part === "requires") ? "requires" : "allows";
  }
  if (type.type === "TSIntersectionType") {
    let found = false;
    for (const part of type.types) {
      const contract = typePropertyContract(part, name, types, seen);
      if (contract === "requires" || contract === "unknown") return contract;
      if (contract === "allows") found = true;
    }
    return found ? "allows" : "missing";
  }
  if (type.type !== "TSTypeReference") return "unknown";
  const nameRef = referenceName(type.typeName);
  if (!nameRef || nameRef === "Promise" || seen.has(nameRef)) return "unknown";
  seen.add(nameRef);
  const alias = lookup(types.aliases, nameRef);
  if (alias) return typePropertyContract(alias, name, types, seen);
  const face = lookup(types.interfaces, nameRef);
  return face ? interfacePropertyContract(face, name, types, seen) : "unknown";
}

function knownPropertyContract(typeNode, name, types) {
  const contract = typePropertyContract(typeNode, name, types, new Set());
  return contract === "missing" ? "unknown" : contract;
}

function functionValueContract(func, types) {
  if (!isFunction(func)) {
    return "unknown";
  }
  if (func.async) return "requires";
  return annotationContract(func.returnType, types);
}

function findClassMember(classNode, name) {
  return classNode.body.body.find(
    (member) =>
      (member.type === "MethodDefinition" || member.type === "PropertyDefinition") &&
      staticKeyName(member) === name,
  );
}

function baseMethodContract(classNode, name, types, seen) {
  if (!classNode?.superClass) return "missing";
  if (seen.has(classNode)) {
    return "unknown";
  }
  seen.add(classNode);
  const base = lookup(types.classes, unwrapExpression(classNode.superClass)?.name);
  if (!base) return "unknown";
  const method = findClassMember(base, name);
  if (!method) return baseMethodContract(base, name, types, seen);
  return method.typeAnnotation
    ? valueContract(method.typeAnnotation, types)
    : functionValueContract(method.value, types);
}

function mentionsClassValue(node, name, seen = new Set()) {
  if (!node || typeof node !== "object" || seen.has(node)) return false;
  seen.add(node);
  if (node.type === "Identifier" && node.name === name) {
    const parent = node.parent;
    const property =
      parent?.type === "MemberExpression" && parent.property === node && !parent.computed;
    return !property;
  }
  for (const [key, value] of Object.entries(node)) {
    if (key === "parent" || !value || typeof value !== "object") continue;
    if (Array.isArray(value)) {
      if (value.some((child) => mentionsClassValue(child, name, seen))) return true;
    } else if (mentionsClassValue(value, name, seen)) return true;
  }
  return false;
}

function extendsClass(classNode, target, types, seen) {
  if (!classNode?.superClass || seen.has(classNode)) return "no";
  seen.add(classNode);
  const superExpr = unwrapExpression(classNode.superClass);
  const targetName = target.id?.name;
  if (superExpr?.type !== "Identifier")
    return targetName && mentionsClassValue(superExpr, targetName) ? "unknown" : "no";
  const found = types.classes.get(superExpr.name);
  if (!found) return "no";
  if (found.ambiguous) return targetName === superExpr.name ? "unknown" : "no";
  if (found.value === target) return "yes";
  return extendsClass(found.value, target, types, seen);
}

function subclassContract(classNode, methodName, types) {
  if (!classNode?.id?.name) return "allows";
  let unknown = false;
  for (const candidate of types.classList) {
    if (candidate === classNode) continue;
    const relation = extendsClass(candidate, classNode, types, new Set());
    if (relation === "no") continue;
    if (relation === "unknown") {
      unknown = true;
      continue;
    }
    if (findClassMember(candidate, methodName)) {
      return "requires";
    }
  }
  return unknown ? "unknown" : "allows";
}

function heritageContract(func, types) {
  const method = func.parent;
  if (method?.type !== "MethodDefinition" && method?.type !== "PropertyDefinition") return "allows";
  const classNode = method.parent.parent;
  const name = staticKeyName(method);
  if (!name) return "unknown";
  const results = [];
  if (method.type === "PropertyDefinition" && method.typeAnnotation) {
    results.push(valueContract(method.typeAnnotation, types));
  }
  const base = baseMethodContract(classNode, name, types, new Set());
  if (base === "missing") results.push(method.override ? "unknown" : "allows");
  else results.push(base);
  // A same-file subclass that declares this method still has to match the
  // async signature. Removing `async` from the base alone is not safe.
  results.push(subclassContract(classNode, name, types));
  for (const implemented of classNode.implements || []) {
    const face = lookup(types.interfaces, unwrapExpression(implemented.expression)?.name);
    if (!face) {
      results.push("unknown");
      continue;
    }
    const property = interfacePropertyContract(face, name, types, new Set());
    results.push(property === "missing" ? "allows" : property);
  }
  return combine(results);
}

function contextualTypeNode(object) {
  const parent = object.parent;
  if (
    parent?.type === "TSAsExpression" ||
    parent?.type === "TSSatisfiesExpression" ||
    parent?.type === "TSTypeAssertion"
  ) {
    return parent.typeAnnotation;
  }
  if (parent?.type === "VariableDeclarator") return parent.id.typeAnnotation || null;
  return null;
}

function contextualContract(func, types) {
  const property = func.parent;
  if (property?.type !== "Property" || property.parent?.type !== "ObjectExpression")
    return "allows";
  const typeNode = contextualTypeNode(property.parent);
  if (!typeNode) return "allows";
  const name = staticKeyName(property);
  return name ? knownPropertyContract(typeNode, name, types) : "unknown";
}

function declarationExported(node) {
  const parent = node?.parent;
  return parent?.type === "ExportNamedDeclaration" || parent?.type === "ExportDefaultDeclaration";
}

function hasLaterWrite(variable, ignored) {
  return variable.references.some(
    (reference) => reference.isWrite() && !reference.init && reference.identifier !== ignored,
  );
}

function boundVariable(func, context) {
  if (func.type === "FunctionDeclaration" && func.id) return resolveVariable(func.id, context);
  const parent = func.parent;
  if (
    parent?.type === "VariableDeclarator" &&
    parent.init === func &&
    parent.id.type === "Identifier"
  ) {
    return resolveVariable(parent.id, context);
  }
  if (
    parent?.type === "AssignmentExpression" &&
    parent.right === func &&
    parent.left.type === "Identifier"
  ) {
    return resolveVariable(parent.left, context);
  }
  return null;
}

function exportContract(func, context) {
  if (declarationExported(func)) return "unknown";
  const variable = boundVariable(func, context);
  if (
    variable?.references.some(
      (reference) => reference.identifier.parent?.type === "ExportSpecifier",
    )
  ) {
    return "unknown";
  }
  const declarator = func.parent?.type === "VariableDeclarator" ? func.parent : null;
  return declarator && declarationExported(declarator.parent) ? "unknown" : "allows";
}

function isInside(node, ancestor) {
  let current = node;
  while (current) {
    if (current === ancestor) return true;
    current = current.parent;
  }
  return false;
}

function isThenObject(node) {
  const member = node.parent;
  if (member?.type !== "MemberExpression" || member.object !== node || member.computed)
    return false;
  const name = propertyName(member.property);
  return (
    PROMISE_USES.has(name) &&
    member.parent?.type === "CallExpression" &&
    member.parent.callee === member
  );
}

function isThenCall(node) {
  return isThenObject(node);
}

function bindingResultContract(declarator, context, types) {
  let result = annotationContract(declarator.id.typeAnnotation, types);
  if (declarationExported(declarator.parent)) result = combine([result, "unknown"]);
  const variable = resolveVariable(declarator.id, context);
  if (!variable || hasLaterWrite(variable)) return combine([result, "unknown"]);
  for (const reference of variable.references) {
    if (reference.init) continue;
    if (isThenObject(reference.identifier)) return "requires";
    const parent = reference.identifier.parent;
    if (parent?.type === "MemberExpression" && parent.object === reference.identifier)
      return combine([result, "unknown"]);
  }
  return result;
}

function returnedPromiseContract(returnStatement, context, types) {
  const owner = findContainingFunction(returnStatement);
  if (owner.async) {
    return "allows";
  } else if (owner.returnType) {
    return annotationContract(owner.returnType, types);
  } else {
    return "unknown";
  }
}

function callResultContract(call, context, types) {
  let current = call;
  while (current) {
    if (isThenCall(current)) return "requires";
    const parent = current.parent;
    if (!parent) return "unknown";
    if (parent.type === "ExpressionStatement") return "allows";
    if (
      parent.type === "VariableDeclarator" &&
      parent.init === current &&
      parent.id.type === "Identifier"
    ) {
      return bindingResultContract(parent, context, types);
    }
    if (parent.type === "AwaitExpression" && parent.argument === current) return "allows";
    if (parent.type === "ReturnStatement" && parent.argument === current)
      return returnedPromiseContract(parent, context, types);
    if (parent.type === "CallExpression" && parent.arguments.includes(current)) return "allows";
    if (
      parent.type === "UnaryExpression" &&
      parent.operator === "void" &&
      parent.argument === current
    )
      return "allows";
    if (
      parent.type === "ChainExpression" ||
      isTransparentExpression(parent) ||
      parent.type === "ConditionalExpression" ||
      parent.type === "LogicalExpression" ||
      parent.type === "SequenceExpression"
    ) {
      current = parent;
      continue;
    }
    return "unknown";
  }
  return "unknown";
}

function argumentParameterContract(argument, context, types) {
  const call = argument.parent;
  const index = call.arguments.indexOf(argument);
  const callee = unwrapExpression(call.callee);
  if (callee?.type !== "Identifier") return "unknown";
  const variable = resolveVariable(callee, context);
  if (!variable || variable.defs.length !== 1) return "unknown";
  const definition = variable.defs[0];
  const target =
    definition.type === "FunctionName"
      ? definition.node
      : definition.type === "Variable" && isFunction(definition.node.init)
        ? definition.node.init
        : null;
  const parameter = target?.params?.[index];
  if (!parameter || parameter.type === "RestElement") return "unknown";
  const binding = parameter.type === "AssignmentPattern" ? parameter.left : parameter;
  return binding?.type === "Identifier"
    ? callableReturnContract(binding.typeAnnotation, types)
    : "unknown";
}

function referenceUseContract(identifier, context, types) {
  const parent = identifier.parent;
  if (!parent || parent.type === "ExportSpecifier") return "unknown";
  if (parent.type === "CallExpression" && parent.callee === identifier)
    return callResultContract(parent, context, types);
  if (parent.type === "CallExpression" && parent.arguments.includes(identifier)) {
    return argumentParameterContract(identifier, context, types);
  }
  return "unknown";
}

function referenceContract(func, context, types) {
  const variable = boundVariable(func, context);
  if (!variable) return directValueContract(func, context, types);
  if (hasLaterWrite(variable, func.id)) return "unknown";
  const results = ["allows"];
  for (const reference of variable.references) {
    if (reference.init || reference.identifier === func.id) continue;
    results.push(referenceUseContract(reference.identifier, context, types));
  }
  return combine(results);
}

function directValueContract(func, context, types) {
  const parent = func.parent;
  if (
    parent?.type === "MethodDefinition" ||
    parent?.type === "PropertyDefinition" ||
    parent?.type === "Property" ||
    (parent?.type === "VariableDeclarator" && parent.init === func)
  ) {
    return "allows";
  }
  if (parent?.type === "CallExpression" && parent.arguments.includes(func)) {
    return argumentParameterContract(func, context, types);
  }
  return "unknown";
}

function isTypePosition(identifier) {
  const parent = identifier.parent;
  return (
    (parent?.type === "TSTypeReference" && parent.typeName === identifier) ||
    (parent?.type === "TSClassImplements" && parent.expression === identifier) ||
    (parent?.type === "TSInterfaceHeritage" && parent.expression === identifier) ||
    (parent?.type === "TSTypeQuery" && parent.exprName === identifier)
  );
}

function memberCallName(identifier) {
  const member = identifier.parent;
  if (
    member?.type !== "MemberExpression" ||
    member.object !== identifier ||
    member.parent?.type !== "CallExpression" ||
    member.parent.callee !== member
  ) {
    return null;
  }
  return staticKeyName(member);
}

function instanceBindingContract(declarator, methodName, context, types) {
  if (declarator.id.type !== "Identifier") return "unknown";
  if (declarationExported(declarator.parent)) return "unknown";
  const variable = resolveVariable(declarator.id, context);
  if (!variable || hasLaterWrite(variable)) return "unknown";
  const results = ["allows"];
  for (const reference of variable.references) {
    if (reference.init) continue;
    const called = memberCallName(reference.identifier);
    if (called === null) return "unknown";
    if (called === methodName) {
      results.push(callResultContract(reference.identifier.parent.parent, context, types));
    }
  }
  return combine(results);
}

function createdValueContract(expression, methodName, context, types) {
  let current = expression;
  while (isTransparentExpression(current.parent)) current = current.parent;
  const parent = current.parent;
  if (parent?.type === "ExpressionStatement") return "allows";
  if (parent?.type !== "VariableDeclarator" || parent.init !== current) return "unknown";
  return instanceBindingContract(parent, methodName, context, types);
}

function thisMemberContract(classNode, methodName, context, types) {
  const results = ["allows"];
  walkAst(context, classNode.body, (node) => {
    if (node.type !== "MemberExpression" || node.object?.type !== "ThisExpression") return;
    const name = staticKeyName(node);
    const called = node.parent?.type === "CallExpression" && node.parent.callee === node;
    if (!called) {
      if (name === null || name === methodName) results.push("unknown");
      return;
    }
    if (name === null) results.push("unknown");
    else if (name === methodName) results.push(callResultContract(node.parent, context, types));
  });
  return combine(results);
}

function variableNamed(scope, name) {
  return scope?.variables.find((item) => item.name === name) || null;
}

function classBindingVariables(classNode, context) {
  // The class body scope binds the name with no outer references. Callers
  // such as `new Repo()` live on the enclosing scope's variable.
  const scope = context.sourceCode.getScope(classNode.id);
  const name = classNode.id.name;
  const inner = variableNamed(scope, name);
  const outer = variableNamed(scope.upper, name);
  return [inner, outer].filter(
    (variable, index, list) => variable && list.indexOf(variable) === index,
  );
}

function isSuperClassName(identifier) {
  let owner = identifier.parent;
  while (isTransparentExpression(owner)) owner = owner.parent;
  if (owner?.type !== "ClassDeclaration" && owner?.type !== "ClassExpression") return false;
  return unwrapExpression(owner.superClass) === identifier;
}

function subclassUseContract(classNode, methodName, context, types, seen) {
  let unknown = false;
  for (const candidate of types.classList) {
    if (candidate === classNode) continue;
    const relation = extendsClass(candidate, classNode, types, new Set());
    if (relation === "no") continue;
    if (relation === "unknown") {
      unknown = true;
      continue;
    }
    // An override must keep the async signature. An inherited method still
    // has callers on the subclass that this class's own `new` sites miss.
    if (findClassMember(candidate, methodName)) {
      return "requires";
    }
    const inherited = classEscapeContract(candidate, methodName, context, types, seen);
    if (inherited === "requires") return "requires";
    if (inherited === "unknown") {
      unknown = true;
    }
  }
  return unknown ? "unknown" : "allows";
}

function classEscapeContract(classNode, methodName, context, types, seen = new Set()) {
  if (declarationExported(classNode)) {
    return "unknown";
  }
  if (
    classNode.parent?.type === "VariableDeclarator" &&
    declarationExported(classNode.parent.parent)
  ) {
    return "unknown";
  }
  if (!classNode.id) return "unknown";
  if (seen.has(classNode)) return "unknown";
  seen.add(classNode);
  const variables = classBindingVariables(classNode, context);
  if (variables.length === 0) return "unknown";
  const results = [thisMemberContract(classNode, methodName, context, types)];
  for (const variable of variables) {
    if (hasLaterWrite(variable, classNode.id)) {
      return "unknown";
    }
    for (const reference of variable.references) {
      if (reference.identifier === classNode.id || isTypePosition(reference.identifier)) continue;
      if (isSuperClassName(reference.identifier)) continue;
      const parent = reference.identifier.parent;
      if (parent?.type === "NewExpression" && parent.callee === reference.identifier) {
        results.push(createdValueContract(parent, methodName, context, types));
        continue;
      }
      return "unknown";
    }
  }
  results.push(subclassUseContract(classNode, methodName, context, types, seen));
  return combine(results);
}

function objectEscapeContract(object, methodName, context, types) {
  let current = object;
  if (
    current.parent?.type === "TSAsExpression" ||
    current.parent?.type === "TSSatisfiesExpression" ||
    current.parent?.type === "TSTypeAssertion"
  ) {
    current = current.parent;
  }
  if (current.parent?.type !== "VariableDeclarator" || current.parent.init !== current)
    return "unknown";
  return instanceBindingContract(current.parent, methodName, context, types);
}

function ownerEscapeContract(func, context, types) {
  const parent = func.parent;
  if (parent?.type === "MethodDefinition" || parent?.type === "PropertyDefinition") {
    const classNode = parent.parent?.type === "ClassBody" ? parent.parent.parent : null;
    const name = staticKeyName(parent);
    return classNode && name ? classEscapeContract(classNode, name, context, types) : "unknown";
  }
  if (parent?.type === "Property" && parent.parent?.type === "ObjectExpression") {
    const name = staticKeyName(parent);
    return name ? objectEscapeContract(parent.parent, name, context, types) : "unknown";
  }
  return "allows";
}

function contractFor(func, context, types) {
  return combine([
    annotationContract(func.returnType, types),
    heritageContract(func, types),
    contextualContract(func, types),
    exportContract(func, context),
    referenceContract(func, context, types),
    ownerEscapeContract(func, context, types),
  ]);
}

module.exports = {
  annotationContract,
  collectTypes,
  contractFor,
  isInside,
  staticKeyName,
};
