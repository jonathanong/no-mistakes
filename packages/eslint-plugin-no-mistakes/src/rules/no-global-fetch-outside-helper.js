"use strict";

const { rule } = require("../helpers");
const helpers = require("./no-global-fetch-outside-helper-helpers");

const {
  collectFetchAliases,
  isGlobalFetchExpression,
  recordAssignmentFetchAliases,
  recordVariableFetchAliases,
  shouldCheckFile,
} = helpers;

module.exports = Object.assign(
  rule(
    {
      type: "problem",
      docs: {
        description: "disallow global fetch outside configured helper paths",
        recommended: false,
      },
      schema: [
        {
          type: "object",
          properties: {
            checkedPathPatterns: { type: "array", items: { type: "string" } },
            allowedPathPatterns: { type: "array", items: { type: "string" } },
          },
          additionalProperties: false,
        },
      ],
      messages: {
        globalFetch:
          "Move global fetch calls into a configured API/client helper so request behavior stays centralized.",
      },
    },
    (context) => {
      const options = context.options?.[0] ?? {};
      if (!shouldCheckFile(context.filename, options)) return {};
      let aliases = new Set();
      const aliasStack = [];

      function pushAliasScope() {
        aliasStack.push(aliases);
        aliases = new Set(aliases);
      }

      function popAliasScope() {
        aliases = aliasStack.pop();
      }

      function recordForInitializer(node) {
        if (node.init?.type === "VariableDeclaration") {
          for (const declaration of node.init.declarations) {
            recordVariableFetchAliases(declaration, context, aliases);
          }
          return;
        }
        if (node.init?.type === "AssignmentExpression") {
          recordAssignmentFetchAliases(node.init, context, aliases);
        }
      }

      return {
        Program(node) {
          collectFetchAliases(node, context, aliases);
        },
        FunctionDeclaration: pushAliasScope,
        "FunctionDeclaration:exit": popAliasScope,
        FunctionExpression: pushAliasScope,
        "FunctionExpression:exit": popAliasScope,
        ArrowFunctionExpression: pushAliasScope,
        "ArrowFunctionExpression:exit": popAliasScope,
        "IfStatement > .consequent": pushAliasScope,
        "IfStatement > .consequent:exit": popAliasScope,
        "IfStatement > .alternate": pushAliasScope,
        "IfStatement > .alternate:exit": popAliasScope,
        ForStatement(node) {
          recordForInitializer(node);
          pushAliasScope();
        },
        "ForStatement:exit": popAliasScope,
        ForInStatement: pushAliasScope,
        "ForInStatement:exit": popAliasScope,
        ForOfStatement: pushAliasScope,
        "ForOfStatement:exit": popAliasScope,
        WhileStatement: pushAliasScope,
        "WhileStatement:exit": popAliasScope,
        DoWhileStatement: pushAliasScope,
        "DoWhileStatement:exit": popAliasScope,
        SwitchStatement: pushAliasScope,
        "SwitchStatement:exit": popAliasScope,
        "TryStatement > .block": pushAliasScope,
        "TryStatement > .block:exit": popAliasScope,
        "TryStatement > .handler": pushAliasScope,
        "TryStatement > .handler:exit": popAliasScope,
        "ClassBody > FieldDefinition[static=false]": pushAliasScope,
        "ClassBody > FieldDefinition[static=false]:exit": popAliasScope,
        "ClassBody > PropertyDefinition[static=false]": pushAliasScope,
        "ClassBody > PropertyDefinition[static=false]:exit": popAliasScope,
        ConditionalExpression: pushAliasScope,
        "ConditionalExpression:exit": popAliasScope,
        LogicalExpression: pushAliasScope,
        "LogicalExpression:exit": popAliasScope,
        VariableDeclarator(node) {
          recordVariableFetchAliases(node, context, aliases);
        },
        AssignmentExpression(node) {
          recordAssignmentFetchAliases(node, context, aliases);
        },
        CallExpression(node) {
          if (!isGlobalFetchExpression(node.callee, context, aliases)) return;
          context.report({ node: node.callee, messageId: "globalFetch" });
        },
      };
    },
  ),
  { __test: helpers },
);
