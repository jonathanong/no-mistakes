"use strict";
const { rule } = require("../helpers");
const { typeAnnotation } = require("../react-node-types");
const { pathAllowed } = require("./module-mock-helpers");
const {
  assertionType,
  compilePatterns,
  isIdentifier,
  memberRootAndProperty,
  objectPropertyName,
  propsFromType,
} = require("./nullable-option-defaults-helpers");
const {
  bindingScope,
  clearNullableBinding,
  clearObjectProps,
  createScope,
  functionScopeVisitors,
  isNullableBinding,
  lexicalScopeVisitors,
  objectProps,
  variableScope,
} = require("./nullable-option-scope");
const { collectTypeProps, createTypeFacts } = require("./nullable-option-type-props");
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
          "Do not default nullable option '{{name}}' with ??, ||, ??=, or ||=. Preserve explicit null and check undefined explicitly.",
      },
    },
    (context) => {
      const options = context.options?.[0] ?? {};
      if (!pathAllowed(context.filename, options)) return {};
      const objectNamePatterns = compilePatterns(options.optionObjectNamePatterns);
      const scopes = [];
      const facts = createTypeFacts();
      const currentScope = () => scopes[scopes.length - 1];
      const pushScope = () => scopes.push(createScope("block"));
      const popScope = () => scopes.pop();
      function enterFunction(node) {
        scopes.push(createScope("function"));
        for (const param of node.params || []) defineParam(param);
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
      const propsForAnnotation = (node) => propsFromType(typeAnnotation(node), facts);
      function defineParam(param) {
        const target = param.type === "AssignmentPattern" ? param.left : param;
        const props = propsForAnnotation(param) || propsForAnnotation(target);
        if (isIdentifier(target)) {
          defineObject(target.name, props);
        } else if (target?.type === "ObjectPattern") {
          definePatternBindings(target, props);
        }
      }
      function definePatternBindings(pattern, props, scope = currentScope(), useExisting = false) {
        for (const property of pattern.properties || []) {
          if (property.type === "RestElement" && isIdentifier(property.argument)) {
            defineObject(property.argument.name, props, scope);
            continue;
          }
          if (property.type !== "Property") continue;
          const name = objectPropertyName(property);
          const value =
            property.value?.type === "AssignmentPattern" ? property.value.left : property.value;
          if (!isIdentifier(value)) continue;
          const targetScope = useExisting ? bindingScope(scopes, value.name) || scope : scope;
          if (name && props?.has(name)) {
            defineNullableBinding(value.name, targetScope);
          } else {
            defineBinding(value.name, targetScope);
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
            props = isIdentifier(node.init) ? objectProps(scopes, node.init.name) : null;
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
          collectTypeProps(node, options, objectNamePatterns, facts);
          scopes.push(createScope("program"));
        },
        "Program:exit": popScope,
        ...functionScopeVisitors(enterFunction, popScope),
        ...lexicalScopeVisitors(pushScope, popScope),
        CatchClause(node) {
          pushScope();
          if (isIdentifier(node.param)) defineBinding(node.param.name);
          if (node.param?.type === "ObjectPattern") definePatternBindings(node.param, null);
        },
        "CatchClause:exit": popScope,
        VariableDeclarator: defineVariable,
        LogicalExpression(node) {
          if (node.operator === "??" || node.operator === "||") reportDefault(node, node.left);
        },
        AssignmentExpression(node) {
          if (node.operator === "=" && node.left.type === "ObjectPattern") {
            const asserted = assertionType(node.right);
            const props = asserted
              ? propsFromType(asserted, facts)
              : isIdentifier(node.right)
                ? objectProps(scopes, node.right.name)
                : null;
            definePatternBindings(node.left, props, currentScope(), true);
            return;
          }
          if (node.operator === "=" && isIdentifier(node.left)) {
            const member = memberRootAndProperty(node.right);
            const props = member ? objectProps(scopes, member.object) : null;
            if (props?.has(member.property)) {
              defineNullableBinding(
                node.left.name,
                bindingScope(scopes, node.left.name) || currentScope(),
              );
            } else {
              clearNullableBinding(scopes, node.left.name);
              clearObjectProps(scopes, node.left.name);
            }
            return;
          }
          if (node.operator === "??=" || node.operator === "||=") reportDefault(node, node.left);
        },
      };
    },
  ),
  {
    __test: {
      ...require("./nullable-option-defaults-helpers"),
      ...require("./nullable-option-scope"),
      ...require("./nullable-option-type-props"),
    },
  },
);
