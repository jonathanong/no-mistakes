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
  assertedObjectFacts,
  classifiedProps,
  destructureMemberFacts,
  memberExpressionStatus,
} = require("./react-nullish-props");

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
      if (scopes[index].bindings.has(name) && !scopes[index][field].has(name)) {
        return alias || null;
      }
      const props = scopes[index][field].get(name);
      if (props) return alias ? new Set([...props, ...alias]) : props;
    }
    return null;
  }

  function objectAliasProps(node) {
    const asserted = assertedObjectFacts(node.init, facts);
    if (asserted) return node.parent.kind === "const" ? asserted : null;
    const init = unwrapSequence(node.init);
    if (node.parent.kind !== "const" || init?.type !== "Identifier") return null;
    const source = resolveVariable(init, context);
    const target = resolveVariable(node.id, context);
    const reassigned = (binding) => binding.references.some((ref) => ref.isWrite() && !ref.init);
    if (!source || source.defs.length !== 1 || reassigned(source) || reassigned(target))
      return null;
    const def = source.defs[0];
    // Sequential name facts must not cross a TDZ shadow or a parameter initializer.
    const initialized =
      def.type === "Parameter"
        ? def.node.body.range[0] < init.range[0]
        : def.type === "Variable" &&
          def.parent.kind === "const" &&
          def.node.range[1] < init.range[0];
    return initialized
      ? { props: objectBindings.get(source), unresolved: unresolvedBindings.get(source) }
      : null;
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
      if (!type) {
        const alias = objectAliasProps(node);
        const binding = resolveVariable(node.id, context);
        if (alias?.props) {
          objectAliases.set(binding, alias.props);
          objectBindings.set(binding, alias.props);
        }
        if (alias?.unresolved) {
          unresolvedAliases.set(binding, alias.unresolved);
          unresolvedBindings.set(binding, alias.unresolved);
        }
      }
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

  function expressionStatus(node) {
    const asserted = assertedType(node);
    if (asserted) return reactTypeStatus(asserted, facts);
    const current = unwrapSequence(node);
    if (isIdentifier(current)) return variableStatus(current.name);
    return current?.type === "MemberExpression"
      ? memberExpressionStatus(current, facts, lookupMembers)
      : "no";
  }

  function enterFunction(node) {
    pushScope("function");
    if (node.type === "FunctionExpression") if (isIdentifier(node.id)) defineBinding(node.id.name);
    for (const param of node.params) defineParam(param);
  }

  return {
    defineVariable,
    enterFunction,
    expressionStatus,
    popScope: () => scopes.pop(),
    pushScope,
    setFacts(next) {
      facts = next;
    },
  };
}

module.exports = { createNullishState };
