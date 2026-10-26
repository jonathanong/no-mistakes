"use strict";

const { rule } = require("../helpers");
const { ruleSuppression } = require("../rule-suppression");
const { literalString } = require("./module-mock-helpers");
const { resolveVariable } = require("./async-target-bindings");
const { createReactNodeFacts, keyName, typeAnnotation, typeName } = require("../react-node-types");

const isObjectPattern = (node) => node && node.type === "ObjectPattern";

const isIdentifier = (node) => node && node.type === "Identifier";

function definePattern(pattern, props, defineBinding, defineReactNode) {
  if (!isObjectPattern(pattern)) return;
  for (const property of pattern.properties) {
    if (property.type !== "Property") continue;
    const name = property.computed
      ? literalString(property.key) || keyName(property.key)
      : keyName(property.key);
    if (isIdentifier(property.value)) {
      defineBinding(property.value.name);
      if (name && props && props.has(name)) defineReactNode(property.value.name);
    } else if (property.value.type === "AssignmentPattern" && isIdentifier(property.value.left)) {
      defineBinding(property.value.left.name);
      if (name && props && props.has(name)) defineReactNode(property.value.left.name);
    }
  }
}

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow nullish coalescing on ReactNode-like values",
      recommended: true,
    },
    schema: [],
    messages: {
      nullish:
        "Do not use ?? with ReactNode values. React renders null, false, and empty values differently; use an explicit undefined check for fallbacks.",
    },
  },
  (context) => {
    const suppressed = ruleSuppression(context, "react-no-nullish-react-node");
    const scopes = [];
    // Loop shadows may share the sequential name scope; alias sources need binding identity.
    const objectBindings = new Map();
    // Keep added alias facts lexical without removing legacy declared-name findings.
    const objectAliases = new Map();
    let facts = { aliases: new Map(), objectProps: new Map(), reactNodeNames: new Set() };

    const currentScope = () => scopes[scopes.length - 1];

    function pushScope(kind) {
      scopes.push({ bindings: new Set(), kind, objectTypes: new Map(), reactNodes: new Set() });
    }

    const popScope = () => scopes.pop();

    function isReactNodeType(type) {
      const name = typeName(type);
      return Boolean(name && (facts.reactNodeNames.has(name) || facts.aliases.get(name) === true));
    }

    function propsForType(type) {
      if (type && type.type === "TSTypeLiteral") {
        const props = new Set();
        for (const member of type.members) {
          if (member.type !== "TSPropertySignature" || !isReactNodeType(typeAnnotation(member))) {
            continue;
          }
          const name = keyName(member.key);
          if (name) props.add(name);
        }
        return props;
      }
      const name = typeName(type);
      return name ? facts.objectProps.get(name) : null;
    }

    function defineReactNode(name, scope = currentScope()) {
      scope.bindings.add(name);
      scope.reactNodes.add(name);
    }

    const defineBinding = (name, scope = currentScope()) => scope.bindings.add(name);

    function defineObjectType(identifier, type, scope = currentScope()) {
      scope.bindings.add(identifier.name);
      const props = propsForType(type);
      if (props && props.size > 0) {
        scope.objectTypes.set(identifier.name, props);
        objectBindings.set(resolveVariable(identifier, context), props);
      }
    }

    function variableScope(node) {
      if (!node.parent || node.parent.kind !== "var") return currentScope();
      return (
        scopes.findLast((scope) => scope.kind === "function" || scope.kind === "program") ||
        currentScope()
      );
    }

    function variableIsReactNode(name) {
      for (let index = scopes.length - 1; index >= 0; index -= 1) {
        if (scopes[index].bindings.has(name)) {
          return scopes[index].reactNodes.has(name);
        }
      }
      return false;
    }

    function objectProps(identifier) {
      const alias = objectAliases.get(resolveVariable(identifier, context));
      const name = identifier.name;
      for (let index = scopes.length - 1; index >= 0; index -= 1) {
        if (scopes[index].bindings.has(name) && !scopes[index].objectTypes.has(name))
          return alias || null;
        const props = scopes[index].objectTypes.get(name);
        if (props) return alias ? new Set([...props, ...alias]) : props;
      }
      return null;
    }

    function objectAliasProps(node) {
      if (node.parent.kind !== "const" || node.init?.type !== "Identifier") return null;
      const source = resolveVariable(node.init, context);
      const target = resolveVariable(node.id, context);
      const reassigned = (binding) => binding.references.some((ref) => ref.isWrite() && !ref.init);
      if (!source || source.defs.length !== 1 || reassigned(source) || reassigned(target))
        return null;
      const def = source.defs[0];
      // Sequential name facts must not cross a TDZ shadow or a parameter initializer.
      const initialized =
        def.type === "Parameter"
          ? def.node.body.range[0] < node.init.range[0]
          : def.type === "Variable" &&
            def.parent.kind === "const" &&
            def.node.range[1] < node.init.range[0];
      return initialized ? objectBindings.get(source) : null;
    }

    function defineParam(param) {
      const target = param.type === "AssignmentPattern" ? param.left : param;
      const type = typeAnnotation(param) || typeAnnotation(target);
      if (isIdentifier(target)) {
        if (isReactNodeType(type)) defineReactNode(target.name);
        defineObjectType(target, type);
      } else if (isObjectPattern(target)) {
        definePattern(target, propsForType(type), defineBinding, defineReactNode);
      }
    }

    function defineVariable(node) {
      const scope = variableScope(node);
      const type = typeAnnotation(node.id);
      if (isIdentifier(node.id)) {
        scope.bindings.add(node.id.name);
        if (isReactNodeType(type) || expressionIsReactNode(node.init)) {
          defineReactNode(node.id.name, scope);
        }
        defineObjectType(node.id, type, scope);
        if (!type) {
          const props = objectAliasProps(node);
          if (props) {
            const binding = resolveVariable(node.id, context);
            objectAliases.set(binding, props);
            objectBindings.set(binding, props);
          }
        }
      } else if (isObjectPattern(node.id)) {
        const initType = isIdentifier(node.init) ? objectProps(node.init) : null;
        definePattern(
          node.id,
          propsForType(type) || initType,
          (name) => defineBinding(name, scope),
          (name) => defineReactNode(name, scope),
        );
      }
    }

    function expressionIsReactNode(node) {
      if (node && node.type === "ChainExpression") return expressionIsReactNode(node.expression);
      if (isIdentifier(node)) return variableIsReactNode(node.name);
      if (node && node.type === "MemberExpression" && isIdentifier(node.object)) {
        const key = node.computed ? literalString(node.property) : keyName(node.property);
        const props = objectProps(node.object);
        return Boolean(key && props && props.has(key));
      }
      return false;
    }

    function enterFunction(node) {
      pushScope("function");
      for (const param of node.params) {
        defineParam(param);
      }
    }

    return {
      Program(node) {
        facts = createReactNodeFacts(node);
        pushScope("program");
      },
      "Program:exit": popScope,
      FunctionDeclaration: enterFunction,
      "FunctionDeclaration:exit": popScope,
      FunctionExpression: enterFunction,
      "FunctionExpression:exit": popScope,
      ArrowFunctionExpression: enterFunction,
      "ArrowFunctionExpression:exit": popScope,
      BlockStatement() {
        pushScope("block");
      },
      "BlockStatement:exit": popScope,
      VariableDeclarator: defineVariable,
      LogicalExpression(node) {
        if (node.operator === "??" && expressionIsReactNode(node.left) && !suppressed(node)) {
          context.report({ node, messageId: "nullish" });
        }
      },
    };
  },
);
