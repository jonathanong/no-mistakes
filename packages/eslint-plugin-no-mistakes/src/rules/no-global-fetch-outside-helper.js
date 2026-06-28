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
      const switchStack = [];
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

      function pushFunctionScope(node) {
        if (isIifeFunction(node)) return;
        functionDepth += 1;
        pushAliasScope();
      }

      function popFunctionScope(node) {
        if (isIifeFunction(node)) return;
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

      function isIifeFunction(node) {
        const parent = node?.parent;
        return parent?.type === "CallExpression" && parent.callee === node;
      }

      function enterSwitch() {
        switchStack.push({ baseAliases: null, baseCleared: null, fallthrough: false });
      }

      function exitSwitch() {
        const state = switchStack.pop();
        if (!state.baseAliases) return;
        aliases = state.baseAliases;
        clearedForwardAliases = state.baseCleared;
      }

      function enterSwitchCase() {
        const state = switchStack.at(-1);
        if (!state) return;
        if (!state.baseAliases) {
          state.baseAliases = aliases;
          state.baseCleared = clearedForwardAliases;
        }
        if (state.fallthrough) return;
        aliases = new Set(state.baseAliases);
        clearedForwardAliases = new Set(state.baseCleared);
      }

      function exitsSwitchCase(node) {
        return (
          node.consequent?.some(
            (child) =>
              child.type === "BreakStatement" ||
              child.type === "ReturnStatement" ||
              child.type === "ThrowStatement",
          ) ?? false
        );
      }

      function exitSwitchCase(node) {
        const state = switchStack.at(-1);
        if (!state) return;
        state.fallthrough = !exitsSwitchCase(node);
        if (!state.fallthrough) {
          aliases = new Set(state.baseAliases);
          clearedForwardAliases = new Set(state.baseCleared);
        }
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
        },
        "ForStatement > .body": pushAliasScope,
        "ForStatement > .body:exit": popAliasScope,
        "ForInStatement > .body": pushAliasScope,
        "ForInStatement > .body:exit": popAliasScope,
        "ForOfStatement > .body": pushAliasScope,
        "ForOfStatement > .body:exit": popAliasScope,
        "WhileStatement > .body": pushAliasScope,
        "WhileStatement > .body:exit": popAliasScope,
        SwitchStatement: enterSwitch,
        "SwitchStatement:exit": exitSwitch,
        SwitchCase: enterSwitchCase,
        "SwitchCase:exit": exitSwitchCase,
        "TryStatement > .block": pushAliasScope,
        "TryStatement > .block:exit": popAliasScope,
        "TryStatement > .handler": pushAliasScope,
        "TryStatement > .handler:exit": popAliasScope,
        "FieldDefinition[static=false] > .value": pushAliasScope,
        "FieldDefinition[static=false] > .value:exit": popAliasScope,
        "PropertyDefinition[static=false] > .value": pushAliasScope,
        "PropertyDefinition[static=false] > .value:exit": popAliasScope,
        "ConditionalExpression > .consequent": pushAliasScope,
        "ConditionalExpression > .consequent:exit": popAliasScope,
        "ConditionalExpression > .alternate": pushAliasScope,
        "ConditionalExpression > .alternate:exit": popAliasScope,
        "LogicalExpression > .right": pushAliasScope,
        "LogicalExpression > .right:exit": popAliasScope,
        VariableDeclarator(node) {
          recordVariableFetchAliases(
            node,
            context,
            aliases,
            clearedForwardAliases,
            activeAliases(),
          );
        },
        AssignmentExpression(node) {
          recordAssignmentFetchAliases(
            node,
            context,
            aliases,
            clearedForwardAliases,
            activeAliases(),
          );
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
