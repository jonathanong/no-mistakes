"use strict";

const { literalString } = require("./module-mock-helpers");
const { unwrapSequence } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { definePattern, keyName, reactTypeStatus, typeAnnotation } = require("../react-node-types");
const { classifiedProps } = require("./react-nullish-props");

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

  function pushScope(kind) {
    scopes.push({
      bindings: new Set(),
      kind,
      objectTypes: new Map(),
      reactNodes: new Set(),
      unresolved: new Set(),
      unresolvedMembers: new Map(),
    });
  }

  function defineReactNode(name, scope = currentScope()) {
    scope.bindings.add(name);
    scope.reactNodes.add(name);
  }

  function defineUnresolved(name, scope = currentScope()) {
    scope.bindings.add(name);
    scope.unresolved.add(name);
  }

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
    return (
      scopes.findLast((scope) => scope.kind === "function" || scope.kind === "program") ||
      currentScope()
    );
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
    if (status === "yes") defineReactNode(name, scope);
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
    const target = param.type === "AssignmentPattern" ? param.left : param;
    const type = typeAnnotation(param) || typeAnnotation(target);
    if (isIdentifier(target)) {
      noteType(target.name, type);
      defineObjectType(target, type);
    } else if (isObjectPattern(target)) {
      const classified = classifiedProps(type, facts);
      definePattern(
        target,
        classified.props,
        defineBinding,
        defineReactNode,
        classified.unresolved,
        defineUnresolved,
      );
    }
  }

  function defineVariable(node) {
    const scope = variableScope(node);
    const type = typeAnnotation(node.id);
    if (isIdentifier(node.id)) {
      scope.bindings.add(node.id.name);
      noteType(node.id.name, type, scope);
      const initStatus = expressionStatus(node.init);
      if (!type && initStatus === "yes") defineReactNode(node.id.name, scope);
      else if (!type && initStatus === "unresolved") defineUnresolved(node.id.name, scope);
      defineObjectType(node.id, type, scope);
      if (!type) {
        const alias = objectAliasProps(node);
        const binding = resolveVariable(node.id, context);
        if (alias?.props) {
          objectAliases.set(binding, alias.props);
          objectBindings.set(binding, alias.props);
        }
        if (alias?.unresolved) unresolvedAliases.set(binding, alias.unresolved);
      }
      return;
    }
    if (!isObjectPattern(node.id)) return;
    const init = unwrapSequence(node.init);
    const initProps = isIdentifier(init) ? lookupMembers(init, "objectTypes") : null;
    const initUnresolved = isIdentifier(init) ? lookupMembers(init, "unresolvedMembers") : null;
    const classified = classifiedProps(type, facts);
    definePattern(
      node.id,
      classified.props || initProps,
      (name) => defineBinding(name, scope),
      (name) => defineReactNode(name, scope),
      classified.unresolved || initUnresolved,
      (name) => defineUnresolved(name, scope),
    );
  }

  function expressionStatus(node) {
    const current = unwrapSequence(node);
    if (isIdentifier(current)) return variableStatus(current.name);
    if (current?.type !== "MemberExpression") return "no";
    const object = unwrapSequence(current.object);
    if (!isIdentifier(object)) return "no";
    const key = current.computed ? literalString(current.property) : keyName(current.property);
    if (key && lookupMembers(object, "objectTypes")?.has(key)) return "yes";
    if (key && lookupMembers(object, "unresolvedMembers")?.has(key)) return "unresolved";
    return "no";
  }

  function enterFunction(node) {
    pushScope("function");
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
