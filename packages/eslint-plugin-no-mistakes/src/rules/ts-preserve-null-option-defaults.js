"use strict";

const { rule } = require("../helpers");
const { typeAnnotation } = require("../react-node-types");
const { pathAllowed } = require("./module-mock-helpers");
const {
  assertionType,
  collectTypeProps,
  compilePatterns,
  createScope,
  isIdentifier,
  isNullableBinding,
  memberRootAndProperty,
  objectProps,
  objectPropertyName,
  propsFromType,
  reportDefaultsInPattern,
  variableScope,
} = require("./nullable-option-defaults-helpers");

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: {
        description: "preserve explicit null in nullable option defaults",
        recommended: false,
      },
      schema: [
        {
          type: "object",
          properties: {
            includePathPatterns: { type: "array", items: { type: "string" } },
            excludePathPatterns: { type: "array", items: { type: "string" } },
            optionObjectNames: { type: "array", items: { type: "string" } },
            optionObjectNamePatterns: { type: "array", items: { type: "string" } },
          },
          additionalProperties: false,
        },
      ],
      messages: {
        default:
          "Do not default nullable option '{{name}}' with ??, ||, ??=, ||=, or destructuring defaults. Preserve explicit null and check undefined explicitly.",
      },
    },
    (context) => {
      const options = context.options?.[0] ?? {};
      if (!pathAllowed(context.filename, options)) return {};
      const objectNamePatterns = compilePatterns(options.optionObjectNamePatterns);
      const scopes = [];
      const facts = { typeProps: new Map() };

      function currentScope() {
        return scopes[scopes.length - 1];
      }

      function pushScope() {
        scopes.push(createScope("block"));
      }

      function popScope() {
        scopes.pop();
      }

      function defineBinding(name, scope = currentScope()) {
        scope.bindings.add(name);
      }

      function defineNullableBinding(name, scope = currentScope()) {
        scope.bindings.add(name);
        scope.nullableBindings.add(name);
      }

      function defineObject(name, props, scope = currentScope()) {
        scope.bindings.add(name);
        if (props && props.size > 0) scope.objectProps.set(name, props);
      }

      function propsForAnnotation(node) {
        return propsFromType(typeAnnotation(node), facts);
      }

      function defineParam(param) {
        const target = param.type === "AssignmentPattern" ? param.left : param;
        const props = propsForAnnotation(param) || propsForAnnotation(target);
        if (isIdentifier(target)) {
          defineObject(target.name, props);
        } else if (target?.type === "ObjectPattern") {
          reportDefaultsInPattern(context, target, props);
          definePatternBindings(target, props);
        }
      }

      function definePatternBindings(pattern, props, scope = currentScope()) {
        for (const property of pattern.properties || []) {
          if (property.type !== "Property") continue;
          const name = objectPropertyName(property);
          const value =
            property.value?.type === "AssignmentPattern" ? property.value.left : property.value;
          if (!isIdentifier(value)) continue;
          if (name && props?.has(name)) {
            defineNullableBinding(value.name, scope);
          } else {
            defineBinding(value.name, scope);
          }
        }
      }

      function defineVariable(node) {
        const scope = variableScope(scopes, currentScope, node);
        let props = propsForAnnotation(node.id);
        const asserted = assertionType(node.init);
        if (isIdentifier(node.id)) {
          if (!props && asserted) {
            props = propsFromType(asserted, facts);
          }
          if (!props && node.init) {
            const member = memberRootAndProperty(node.init);
            const objProps = member ? objectProps(scopes, member.object) : null;
            if (objProps?.has(member.property)) {
              defineNullableBinding(node.id.name, scope);
              return;
            }
          }
          defineObject(node.id.name, props, scope);
          return;
        }
        if (node.id?.type === "ObjectPattern") {
          let initProps = isIdentifier(node.init) ? objectProps(scopes, node.init.name) : null;
          if (!initProps && asserted) {
            initProps = propsFromType(asserted, facts);
          }
          const finalProps = props || initProps;
          reportDefaultsInPattern(context, node.id, finalProps);
          definePatternBindings(node.id, finalProps, scope);
        }
      }

      function reportDefault(node, target) {
        if (isIdentifier(target)) {
          if (isNullableBinding(scopes, target.name)) {
            context.report({ node, messageId: "default", data: { name: target.name } });
          }
          return;
        }
        const member = memberRootAndProperty(target);
        if (!member) return;
        const props = objectProps(scopes, member.object);
        if (!props?.has(member.property)) return;
        context.report({ node, messageId: "default", data: { name: member.property } });
      }

      return {
        Program(node) {
          collectTypeProps(node, options, objectNamePatterns, facts.typeProps);
          scopes.push(createScope("program"));
        },
        "Program:exit": popScope,
        FunctionDeclaration(node) {
          scopes.push(createScope("function"));
          for (const param of node.params || []) defineParam(param);
        },
        "FunctionDeclaration:exit": popScope,
        FunctionExpression(node) {
          scopes.push(createScope("function"));
          for (const param of node.params || []) defineParam(param);
        },
        "FunctionExpression:exit": popScope,
        ArrowFunctionExpression(node) {
          scopes.push(createScope("function"));
          for (const param of node.params || []) defineParam(param);
        },
        "ArrowFunctionExpression:exit": popScope,
        BlockStatement() {
          pushScope();
        },
        "BlockStatement:exit": popScope,
        VariableDeclarator: defineVariable,
        LogicalExpression(node) {
          if (node.operator === "??" || node.operator === "||") reportDefault(node, node.left);
        },
        AssignmentExpression(node) {
          if (node.operator === "??=" || node.operator === "||=") reportDefault(node, node.left);
        },
      };
    },
  ),
  {
    __test: require("./nullable-option-defaults-helpers"),
  },
);
