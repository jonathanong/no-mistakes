"use strict";

const { unwrapSequence } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const {
  assertedType,
  bindPatternNames,
  definePattern,
  reactTypeStatus,
  typeAnnotation,
} = require("../react-node-types");
const {
  classifiedProps,
  destructureMemberFacts,
  memberExpressionStatus,
} = require("./react-nullish-props");
const { notePair, objectAliasProps } = require("./react-nullish-loop-members");
const { laterBindingStatus } = require("./react-nullish-later-status");

const isObjectPattern = (node) => node && node.type === "ObjectPattern";
const isIdentifier = (node) => node && node.type === "Identifier";

function createNullishState(context) {
  const scopes = [];
  // Loop shadows may share the sequential name scope; alias sources need binding identity.
  const objectBindings = new Map();
  const unresolvedBindings = new Map();
  // Keep added alias facts lexical without removing legacy declared-name findings.
  const objectAliases = new Map();
  const unresolvedAliases = new Map();
  let facts = {
    aliases: new Map(),
    objectProps: new Map(),
    reactNodeNames: new Set(),
    unresolvedNamespaces: new Set(),
    unresolvedTypes: new Set(),
  };

  const currentScope = () => scopes[scopes.length - 1];
  const defineBinding = (name, scope = currentScope()) => scope.bindings.add(name);

  function pushScope(kind, pattern) {
    scopes.push({
      bindings: new Set(),
      kind,
      objectTypes: new Map(),
      reactNodes: new Set(),
      unresolved: new Set(),
      unresolvedMembers: new Map(),
    });
    if (pattern) bindPatternNames(pattern, defineBinding);
  }

  function defineNamed(field, name, scope = currentScope()) {
    scope.bindings.add(name);
    scope[field].add(name);
  }

  const defineUnresolved = (name, scope) => defineNamed("unresolved", name, scope);

  function defineObjectType(identifier, type, scope = currentScope()) {
    scope.bindings.add(identifier.name);
    const classified = classifiedProps(type, facts);
    if (classified.props && classified.props.size > 0) {
      scope.objectTypes.set(identifier.name, classified.props);
      objectBindings.set(resolveVariable(identifier, context), classified.props);
    }
    if (classified.unresolved && classified.unresolved.size > 0) {
      scope.unresolvedMembers.set(identifier.name, classified.unresolved);
      unresolvedBindings.set(resolveVariable(identifier, context), classified.unresolved);
    }
  }

  function variableScope(node) {
    if (!node.parent || node.parent.kind !== "var") return currentScope();
    const outer = (scope) => scope.kind === "function" || scope.kind === "program";
    return scopes.findLast(outer) ?? currentScope();
  }

  function variableStatus(name) {
    for (let index = scopes.length - 1; index >= 0; index -= 1) {
      if (!scopes[index].bindings.has(name)) continue;
      if (scopes[index].reactNodes.has(name)) return "yes";
      return scopes[index].unresolved.has(name) ? "unresolved" : "no";
    }
    return "no";
  }

  function noteType(name, type, scope = currentScope()) {
    const status = reactTypeStatus(type, facts);
    if (status === "yes") defineNamed("reactNodes", name, scope);
    else if (status === "unresolved") defineUnresolved(name, scope);
  }

  function lookupMembers(identifier, field) {
    const aliases = field === "objectTypes" ? objectAliases : unresolvedAliases;
    const alias = aliases.get(resolveVariable(identifier, context));
    const name = identifier.name;
    for (let index = scopes.length - 1; index >= 0; index -= 1) {
      if (scopes[index].bindings.has(name) && !scopes[index][field].has(name)) return alias || null;
      const props = scopes[index][field].get(name);
      if (props) return alias ? new Set([...props, ...alias]) : props;
    }
    return null;
  }

  function defineParam(param) {
    const target = (param.parameter ?? param).left ?? param.parameter ?? param;
    const type = typeAnnotation(param) || typeAnnotation(target);
    if (isIdentifier(target)) {
      noteType(target.name, type);
      defineObjectType(target, type);
    } else if (target.type === "ArrayPattern" || target.type === "RestElement") {
      bindPatternNames(target, defineBinding);
    } else if (isObjectPattern(target)) {
      const members = destructureMemberFacts(type, null, facts, lookupMembers);
      definePattern(
        target,
        members.props,
        defineBinding,
        (name) => defineNamed("reactNodes", name),
        members.unresolved,
        defineUnresolved,
        context,
      );
      if (members.allUnresolved) bindPatternNames(target, defineUnresolved);
    }
  }

  function finishLoop() {
    scopes.pop();
  }

  function noteObjectAlias(node) {
    const alias = objectAliasProps(node, facts, context, objectBindings, unresolvedBindings);
    const binding = resolveVariable(node.id, context);
    notePair(objectBindings, objectAliases, binding, alias?.props);
    notePair(unresolvedBindings, unresolvedAliases, binding, alias?.unresolved);
  }

  function defineVariable(node) {
    const scope = variableScope(node);
    const type = typeAnnotation(node.id);
    if (isIdentifier(node.id)) {
      scope.bindings.add(node.id.name);
      noteType(node.id.name, type, scope);
      const initStatus = expressionStatus(node.init);
      if (!type && initStatus === "yes") defineNamed("reactNodes", node.id.name, scope);
      else if (!type && initStatus === "unresolved") defineUnresolved(node.id.name, scope);
      defineObjectType(node.id, type, scope);
      if (!type) noteObjectAlias(node);
      return;
    }
    if (node.id.type === "ArrayPattern")
      return bindPatternNames(node.id, (name) => defineBinding(name, scope));
    if (!isObjectPattern(node.id)) return;
    const members = destructureMemberFacts(type, node.init, facts, lookupMembers);
    definePattern(
      node.id,
      members.props,
      (name) => defineBinding(name, scope),
      (name) => defineNamed("reactNodes", name, scope),
      members.unresolved,
      (name) => defineUnresolved(name, scope),
      context,
    );
    if (members.allUnresolved) bindPatternNames(node.id, (name) => defineUnresolved(name, scope));
  }

  function expressionStatus(node, seen = new Set()) {
    const asserted = assertedType(node);
    if (asserted) return reactTypeStatus(asserted, facts);
    const current = unwrapSequence(node);
    if (isIdentifier(current)) {
      const later = laterBindingStatus(
        current,
        context,
        (type) => reactTypeStatus(type, facts),
        expressionStatus,
        facts,
        seen,
      );
      return later ?? variableStatus(current.name);
    }
    if (current?.type !== "MemberExpression") return "no";
    return memberExpressionStatus(current, facts, lookupMembers);
  }

  function enterFunction(node) {
    if (node.type === "FunctionDeclaration") if (isIdentifier(node.id)) defineBinding(node.id.name);
    pushScope("function");
    if (node.type === "FunctionExpression") if (isIdentifier(node.id)) defineBinding(node.id.name);
    for (const param of node.params) defineParam(param);
  }

  return {
    defineVariable,
    enterFunction,
    expressionStatus,
    finishLoop,
    popScope: () => scopes.pop(),
    pushScope,
    setFacts(next) {
      facts = next;
      facts.context = context;
    },
    shadowName: (name) => currentScope()?.bindings.add(name),
  };
}

module.exports = { createNullishState };
