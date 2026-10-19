"use strict";

const { isTransparentExpression, unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const {
  declarationExported,
  exportContract,
  hasLaterWrite,
} = require("./postgres-no-async-sql-builder-bindings");
const { callResultContract, referenceContract } = require("./postgres-no-async-sql-builder-calls");
const {
  extendsClass,
  findClassMember,
  heritageContract,
} = require("./postgres-no-async-sql-builder-heritage");
const { contextualContract } = require("./postgres-no-async-sql-builder-property");
const {
  annotationContract,
  combine,
  staticKeyName,
  walkAst,
} = require("./postgres-no-async-sql-builder-types");

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
  contractFor,
};
