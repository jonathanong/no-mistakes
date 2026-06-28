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
      const forwardAliases = new Set();
      let clearedForwardAliases = new Set();
      const aliasStack = [];
      const clearedAliasStack = [];
      let functionDepth = 0;

      function pushAliasScope() {
        aliasStack.push(aliases);
        clearedAliasStack.push(clearedForwardAliases);
        aliases = new Set(aliases);
        clearedForwardAliases = new Set(clearedForwardAliases);
      }

      function popAliasScope() {
        aliases = aliasStack.pop();
        clearedForwardAliases = clearedAliasStack.pop();
      }

      function pushFunctionScope() {
        functionDepth += 1;
        pushAliasScope();
      }

      function popFunctionScope() {
        popAliasScope();
        functionDepth -= 1;
      }

      function activeAliases() {
        if (functionDepth === 0) return aliases;
        const active = new Set(forwardAliases);
        for (const alias of clearedForwardAliases) active.delete(alias);
        for (const alias of aliases) active.add(alias);
        return active;
      }

      function recordForInitializer(node) {
        if (node.init?.type === "VariableDeclaration") {
          for (const declaration of node.init.declarations) {
            recordVariableFetchAliases(declaration, context, aliases, clearedForwardAliases);
          }
          return;
        }
        if (node.init?.type === "AssignmentExpression") {
          recordAssignmentFetchAliases(node.init, context, aliases, clearedForwardAliases);
        }
      }

      return {
        Program(node) {
          collectFetchAliases(node, context, forwardAliases);
        },
        FunctionDeclaration: pushFunctionScope,
        "FunctionDeclaration:exit": popFunctionScope,
        FunctionExpression: pushFunctionScope,
        "FunctionExpression:exit": popFunctionScope,
        ArrowFunctionExpression: pushFunctionScope,
        "ArrowFunctionExpression:exit": popFunctionScope,
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
        SwitchCase: pushAliasScope,
        "SwitchCase:exit": popAliasScope,
        "TryStatement > .block": pushAliasScope,
        "TryStatement > .block:exit": popAliasScope,
        "TryStatement > .handler": pushAliasScope,
        "TryStatement > .handler:exit": popAliasScope,
        "ClassBody > FieldDefinition[static=false]": pushAliasScope,
        "ClassBody > FieldDefinition[static=false]:exit": popAliasScope,
        "ClassBody > PropertyDefinition[static=false]": pushAliasScope,
        "ClassBody > PropertyDefinition[static=false]:exit": popAliasScope,
        "ConditionalExpression > .consequent": pushAliasScope,
        "ConditionalExpression > .consequent:exit": popAliasScope,
        "ConditionalExpression > .alternate": pushAliasScope,
        "ConditionalExpression > .alternate:exit": popAliasScope,
        "LogicalExpression > .right": pushAliasScope,
        "LogicalExpression > .right:exit": popAliasScope,
        VariableDeclarator(node) {
          recordVariableFetchAliases(node, context, aliases, clearedForwardAliases);
        },
        AssignmentExpression(node) {
          recordAssignmentFetchAliases(node, context, aliases, clearedForwardAliases);
        },
        CallExpression(node) {
          if (!isGlobalFetchExpression(node.callee, context, activeAliases())) return;
          context.report({ node: node.callee, messageId: "globalFetch" });
        },
      };
    },
  ),
  { __test: helpers },
);
