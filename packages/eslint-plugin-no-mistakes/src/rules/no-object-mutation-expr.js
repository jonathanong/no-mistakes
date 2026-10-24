"use strict";

const api = require("./no-object-mutation-runtime");
const {
  visitArray,
  visitAssignment,
  visitObject,
  walkChildren,
} = require("./no-object-mutation-build");
const { visitCall, visitIdentifier, visitNew } = require("./no-object-mutation-call");
const { memberKey, unwrap } = require("./no-object-mutation-syntax");
const { joinValues, readMember, shared, unknown } = require("./no-object-mutation-values");
const { considerWrite } = require("./no-object-mutation-write");

function visitExpression(state, node, scope, role) {
  switch (node.type) {
    case "Identifier":
      return visitIdentifier(state, node, scope, role);
    case "ThisExpression":
      if (scope.receiver) {
        return scope.receiver;
      }
      return { type: "this", inClass: Boolean(scope.inClass) };
    case "Super":
    case "Literal":
    case "TemplateElement":
    case "JSXText":
    case "JSXIdentifier":
    case "JSXEmptyExpression":
      return unknown();
    case "TemplateLiteral":
      for (const expression of node.expressions) {
        api.visit(state, expression, scope, "read");
      }
      return unknown();
    case "TaggedTemplateExpression":
      api.visit(state, node.tag, scope, "read");
      for (const expression of node.quasi.expressions) {
        api.visit(state, expression, scope, "escape");
      }
      return unknown();
    case "YieldExpression":
      if (node.argument) {
        api.visit(state, node.argument, scope, "escape");
      }
      return unknown();
    case "AwaitExpression":
      return api.visit(state, node.argument, scope, "read");
    case "UnaryExpression":
      if (node.operator === "delete") {
        api.visit(state, node.argument, scope, "read");
        return unknown();
      }
      return api.visit(state, node.argument, scope, "read");
    case "UpdateExpression": {
      const member = unwrap(node.argument);
      if (member?.type === "MemberExpression") {
        if (member.computed) {
          api.visit(state, member.property, scope, "read");
        }
        considerWrite(state, api.visit(state, member.object, scope, "read"), node, {
          viaCall: false,
          objectNode: member,
          key: memberKey(member),
          scope,
        });
      } else {
        api.visit(state, node.argument, scope, "read");
      }
      return unknown();
    }
    case "BinaryExpression":
      api.visit(state, node.left, scope, "read");
      api.visit(state, node.right, scope, "read");
      return unknown();
    case "LogicalExpression":
      // && yields the left only when that side is falsy, so it cannot be the object.
      if (node.operator === "&&") {
        api.visit(state, node.left, scope, "read");
        return api.visit(state, node.right, scope, role);
      }
      return joinValues(
        api.visit(state, node.left, scope, role),
        api.visit(state, node.right, scope, role),
      );
    case "ConditionalExpression":
      api.visit(state, node.test, scope, "read");
      return joinValues(
        api.visit(state, node.consequent, scope, role),
        api.visit(state, node.alternate, scope, role),
      );
    case "SequenceExpression": {
      let tag = unknown();
      for (const expression of node.expressions) {
        tag = api.visit(state, expression, scope, role);
      }
      return tag;
    }
    case "AssignmentExpression":
      return visitAssignment(state, node, scope);
    case "MemberExpression": {
      const member = readMember(
        api.visit(state, node.object, scope, "read"),
        memberKey(node),
        node.object,
      );
      if (role === "escape") api.releaseEscaping(state, member);
      return member;
    }
    case "ObjectExpression":
      return visitObject(state, node, scope, role);
    case "ArrayExpression":
      return visitArray(state, node, scope, role);
    case "NewExpression":
      return visitNew(state, node, scope, role);
    case "CallExpression":
      return visitCall(state, node, scope, role);
    case "ImportExpression":
      api.visit(state, node.source, scope, "read");
      return unknown();
    case "MetaProperty":
      if (node.meta?.name === "import" && node.property?.name === "meta") {
        return shared("module");
      }
      return unknown();
    case "JSXElement":
    case "JSXFragment":
    case "JSXAttribute":
    case "JSXOpeningElement":
    case "JSXClosingElement":
      walkChildren(state, node, scope);
      return unknown();
    case "JSXSpreadAttribute":
      return api.visit(state, node.argument, scope, "escape");
    case "JSXSpreadChild":
      return api.visit(state, node.expression, scope, "escape");
    case "JSXExpressionContainer":
      return api.visit(state, node.expression, scope, "escape");
    default:
      return undefined;
  }
}

module.exports = { visitExpression };
