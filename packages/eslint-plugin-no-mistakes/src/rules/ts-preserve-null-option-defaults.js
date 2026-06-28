"use strict";

const { rule } = require("../helpers");
const { typeAnnotation } = require("../react-node-types");
const { pathAllowed } = require("./module-mock-helpers");
const {
  compilePatterns,
  isIdentifier,
  memberRootAndProperty,
  nullablePropsFromMembers,
  objectPropertyName,
  optionTypeAllowed,
  propsFromType,
} = require("./nullable-option-defaults-helpers");

function reportDefaultsInPattern(context, pattern, props) {
  if (!pattern || pattern.type !== "ObjectPattern" || !props) return;
  for (const property of pattern.properties || []) {
    const name = objectPropertyName(property);
    if (!name || !props.has(name)) continue;
    if (property.value?.type === "AssignmentPattern") {
      context.report({ node: property.value, messageId: "default", data: { name } });
    }
  }
}

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
        scopes.push({ bindings: new Set(), objectProps: new Map() });
      }

      function popScope() {
        scopes.pop();
      }

      function defineBinding(name, scope = currentScope()) {
        scope.bindings.add(name);
      }

      function defineObject(name, props, scope = currentScope()) {
        scope.bindings.add(name);
        if (props && props.size > 0) scope.objectProps.set(name, props);
      }

      function variableScope(node) {
        if (!node.parent || node.parent.kind !== "var") return currentScope();
        return (
          scopes.findLast((scope) => scope.kind === "function" || scope.kind === "program") ||
          currentScope()
        );
      }

      function objectProps(name) {
        for (let index = scopes.length - 1; index >= 0; index -= 1) {
          if (scopes[index].bindings.has(name) && !scopes[index].objectProps.has(name)) return null;
          const props = scopes[index].objectProps.get(name);
          if (props) return props;
        }
        return null;
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
        }
      }

      function defineVariable(node) {
        const scope = variableScope(node);
        const props = propsForAnnotation(node.id);
        if (isIdentifier(node.id)) {
          defineObject(node.id.name, props, scope);
          return;
        }
        if (node.id?.type === "ObjectPattern") {
          const initProps = isIdentifier(node.init) ? objectProps(node.init.name) : null;
          reportDefaultsInPattern(context, node.id, props || initProps);
          for (const property of node.id.properties || []) {
            if (property.type === "Property" && isIdentifier(property.value)) {
              defineBinding(property.value.name, scope);
            }
          }
        }
      }

      function reportMemberDefault(node, target) {
        const member = memberRootAndProperty(target);
        if (!member) return;
        const props = objectProps(member.object);
        if (!props?.has(member.property)) return;
        context.report({ node, messageId: "default", data: { name: member.property } });
      }

      return {
        Program(node) {
          for (const statement of node.body || []) {
            const declaration =
              statement.type === "ExportNamedDeclaration" && statement.declaration
                ? statement.declaration
                : statement;
            if (
              declaration.type === "TSInterfaceDeclaration" &&
              optionTypeAllowed(declaration.id.name, options, objectNamePatterns)
            ) {
              facts.typeProps.set(
                declaration.id.name,
                nullablePropsFromMembers(declaration.body.body),
              );
            }
            if (
              declaration.type === "TSTypeAliasDeclaration" &&
              optionTypeAllowed(declaration.id.name, options, objectNamePatterns) &&
              declaration.typeAnnotation.type === "TSTypeLiteral"
            ) {
              facts.typeProps.set(
                declaration.id.name,
                nullablePropsFromMembers(declaration.typeAnnotation.members),
              );
            }
          }
          scopes.push({ bindings: new Set(), kind: "program", objectProps: new Map() });
        },
        "Program:exit": popScope,
        FunctionDeclaration(node) {
          scopes.push({ bindings: new Set(), kind: "function", objectProps: new Map() });
          for (const param of node.params || []) defineParam(param);
        },
        "FunctionDeclaration:exit": popScope,
        FunctionExpression(node) {
          scopes.push({ bindings: new Set(), kind: "function", objectProps: new Map() });
          for (const param of node.params || []) defineParam(param);
        },
        "FunctionExpression:exit": popScope,
        ArrowFunctionExpression(node) {
          scopes.push({ bindings: new Set(), kind: "function", objectProps: new Map() });
          for (const param of node.params || []) defineParam(param);
        },
        "ArrowFunctionExpression:exit": popScope,
        BlockStatement() {
          pushScope();
        },
        "BlockStatement:exit": popScope,
        VariableDeclarator: defineVariable,
        LogicalExpression(node) {
          if (node.operator === "??" || node.operator === "||")
            reportMemberDefault(node, node.left);
        },
        AssignmentExpression(node) {
          if (node.operator === "??=" || node.operator === "||=")
            reportMemberDefault(node, node.left);
        },
      };
    },
  ),
  {
    __test: require("./nullable-option-defaults-helpers"),
  },
);
