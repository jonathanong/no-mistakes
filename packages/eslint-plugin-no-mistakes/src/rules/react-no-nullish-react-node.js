"use strict";

const { rule } = require("../helpers");
const { createReactNodeFacts, keyName, typeAnnotation, typeName } = require("../react-node-types");

function isObjectPattern(node) {
  return node && node.type === "ObjectPattern";
}

function isIdentifier(node) {
  return node && node.type === "Identifier";
}

function definePattern(pattern, props, defineBinding, defineReactNode) {
  if (!isObjectPattern(pattern)) return;
  for (const property of pattern.properties || []) {
    if (property.type !== "Property") continue;
    const name = keyName(property.key);
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
    const scopes = [];
    let facts = { aliases: new Map(), objectProps: new Map(), reactNodeNames: new Set() };

    function currentScope() {
      return scopes[scopes.length - 1];
    }

    function pushScope() {
      scopes.push({ bindings: new Set(), objectTypes: new Map(), reactNodes: new Set() });
    }

    function popScope() {
      scopes.pop();
    }

    function isReactNodeType(type) {
      const name = typeName(type);
      return Boolean(name && (facts.reactNodeNames.has(name) || facts.aliases.get(name) === true));
    }

    function propsForType(type) {
      const name = typeName(type);
      return name ? facts.objectProps.get(name) : null;
    }

    function defineReactNode(name) {
      currentScope().bindings.add(name);
      currentScope().reactNodes.add(name);
    }

    function defineBinding(name) {
      currentScope().bindings.add(name);
    }

    function defineObjectType(name, type) {
      currentScope().bindings.add(name);
      const props = propsForType(type);
      if (props && props.size > 0) {
        currentScope().objectTypes.set(name, props);
      }
    }

    function variableIsReactNode(name) {
      for (let index = scopes.length - 1; index >= 0; index -= 1) {
        if (scopes[index].bindings.has(name)) {
          return scopes[index].reactNodes.has(name);
        }
      }
      return false;
    }

    function objectProps(name) {
      for (let index = scopes.length - 1; index >= 0; index -= 1) {
        if (scopes[index].bindings.has(name) && !scopes[index].objectTypes.has(name)) return null;
        const props = scopes[index].objectTypes.get(name);
        if (props) return props;
      }
      return null;
    }

    function defineParam(param) {
      const type = typeAnnotation(param);
      if (isIdentifier(param)) {
        currentScope().bindings.add(param.name);
        if (isReactNodeType(type)) defineReactNode(param.name);
        defineObjectType(param.name, type);
      } else if (isObjectPattern(param)) {
        definePattern(param, propsForType(type), defineBinding, defineReactNode);
      }
    }

    function defineVariable(node) {
      const type = typeAnnotation(node.id);
      if (isIdentifier(node.id)) {
        currentScope().bindings.add(node.id.name);
        if (isReactNodeType(type)) defineReactNode(node.id.name);
        defineObjectType(node.id.name, type);
      } else if (isObjectPattern(node.id)) {
        const initType = isIdentifier(node.init) ? objectProps(node.init.name) : null;
        definePattern(node.id, propsForType(type) || initType, defineBinding, defineReactNode);
      }
    }

    function expressionIsReactNode(node) {
      if (isIdentifier(node)) return variableIsReactNode(node.name);
      if (node && node.type === "MemberExpression" && !node.computed && isIdentifier(node.object)) {
        const props = objectProps(node.object.name);
        return Boolean(props && props.has(keyName(node.property)));
      }
      return false;
    }

    function enterFunction(node) {
      pushScope();
      for (const param of node.params || []) {
        defineParam(param);
      }
    }

    return {
      Program(node) {
        facts = createReactNodeFacts(node);
        pushScope();
      },
      "Program:exit": popScope,
      FunctionDeclaration: enterFunction,
      "FunctionDeclaration:exit": popScope,
      FunctionExpression: enterFunction,
      "FunctionExpression:exit": popScope,
      ArrowFunctionExpression: enterFunction,
      "ArrowFunctionExpression:exit": popScope,
      BlockStatement: pushScope,
      "BlockStatement:exit": popScope,
      VariableDeclarator: defineVariable,
      LogicalExpression(node) {
        if (node.operator === "??" && expressionIsReactNode(node.left)) {
          context.report({ node, messageId: "nullish" });
        }
      },
    };
  },
);
