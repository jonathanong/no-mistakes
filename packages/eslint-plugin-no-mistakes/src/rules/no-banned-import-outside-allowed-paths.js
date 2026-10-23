"use strict";

const { createInvocationChecker } = require("./no-banned-import-outside-allowed-paths-calls");
const { rule } = require("../helpers");
const {
  normalizeBannedImports,
  shouldCheckFile,
} = require("./no-banned-import-outside-allowed-paths-config");
const {
  checkDefaultExportLeak,
  checkExportLeaks,
  seedImportTags,
} = require("./no-banned-import-outside-allowed-paths-imports");
const {
  collectBannedAliases,
  recordAssignmentTag,
  recordVariableTag,
} = require("./no-banned-import-outside-allowed-paths-aliases");
const { createAliasScopeTracker } = require("./no-banned-import-outside-allowed-paths-scopes");

module.exports = rule(
  {
    type: "problem",
    docs: {
      description: "disallow banned capability imports outside allowed paths",
      recommended: false,
    },
    schema: [
      {
        type: "object",
        properties: {
          checkedPathPatterns: { type: "array", items: { type: "string" } },
          allowedPathPatterns: { type: "array", items: { type: "string" } },
          bannedImports: {
            type: "array",
            items: {
              type: "object",
              properties: {
                module: { type: "string" },
                names: { type: "array", items: { type: "string" }, minItems: 1 },
              },
              required: ["module", "names"],
              additionalProperties: false,
            },
          },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      bannedImport:
        'Reachable use of banned import "{{name}}" from "{{module}}" outside allowed paths. Move it into an allowed helper path.',
      bannedReExport:
        'Do not re-export banned import "{{name}}" from "{{module}}"; it stays reachable outside allowed paths.',
    },
  },
  (context) => {
    const options = context.options?.[0] ?? {};
    if (!shouldCheckFile(context.filename, options)) return {};
    const config = normalizeBannedImports(options.bannedImports);
    if (config.size === 0) return {};

    const scopes = createAliasScopeTracker();
    const forwardAliases = new Map();
    let functionDepth = 0;

    function isIifeFunction(node) {
      const parent = node?.parent;
      return parent?.type === "CallExpression" && parent.callee === node;
    }

    function pushFunctionScope(node) {
      if (isIifeFunction(node)) return;
      functionDepth += 1;
      scopes.push();
    }

    function popFunctionScope(node) {
      if (isIifeFunction(node)) return;
      scopes.pop();
      functionDepth -= 1;
    }

    // Functions can reach hoisted module tags; module execution keeps source order.
    function activeAliases() {
      if (functionDepth === 0) return scopes.aliases;
      const active = new Map(forwardAliases);
      for (const variable of scopes.clearedForwardAliases) active.delete(variable);
      for (const [variable, tag] of scopes.aliases) active.set(variable, tag);
      return active;
    }

    const checkInvocation = createInvocationChecker(context, config, activeAliases);

    return {
      Program(node) {
        seedImportTags(node, context, config, scopes.aliases);
        for (const [variable, tag] of scopes.aliases) forwardAliases.set(variable, tag);
        collectBannedAliases(node, context, forwardAliases, config);
      },
      FunctionDeclaration: pushFunctionScope,
      "FunctionDeclaration:exit": popFunctionScope,
      FunctionExpression: pushFunctionScope,
      "FunctionExpression:exit": popFunctionScope,
      ArrowFunctionExpression: pushFunctionScope,
      "ArrowFunctionExpression:exit": popFunctionScope,
      "IfStatement > .consequent": scopes.push,
      "IfStatement > .consequent:exit": scopes.pop,
      "IfStatement > .alternate": scopes.push,
      "IfStatement > .alternate:exit": scopes.pop,
      "ForStatement > .body": scopes.push,
      "ForStatement > .body:exit": scopes.pop,
      "ForInStatement > .body": scopes.push,
      "ForInStatement > .body:exit": scopes.pop,
      "ForOfStatement > .body": scopes.push,
      "ForOfStatement > .body:exit": scopes.pop,
      "WhileStatement > .body": scopes.push,
      "WhileStatement > .body:exit": scopes.pop,
      SwitchStatement: scopes.enterSwitch,
      "SwitchStatement:exit": scopes.exitSwitch,
      SwitchCase: scopes.enterSwitchCase,
      "SwitchCase:exit": scopes.exitSwitchCase,
      "TryStatement > .block": scopes.push,
      "TryStatement > .block:exit": scopes.pop,
      "TryStatement > .handler": scopes.push,
      "TryStatement > .handler:exit": scopes.pop,
      "FieldDefinition[static=false] > .value": scopes.push,
      "FieldDefinition[static=false] > .value:exit": scopes.pop,
      "PropertyDefinition[static=false] > .value": scopes.push,
      "PropertyDefinition[static=false] > .value:exit": scopes.pop,
      // A ternary's or `&&`/`||`'s conditionally-executed operand can itself
      // be a bare AssignmentExpression (no wrapping statement), unlike an
      // `if`/loop/switch/try body, which is always a Statement. Pushing on
      // that operand's own field selector would race the operand's own
      // enter listener (both fire on the same node, and the plain-type
      // listener runs first), so the push/pop below is keyed to the
      // guaranteed-unconditional sibling's exit and the container's exit
      // instead, which always bracket the conditional operand's own enter
      // and exit regardless of listener-specificity ordering.
      "ConditionalExpression > .test:exit": scopes.push,
      "ConditionalExpression > .consequent:exit": scopes.resetBranch,
      "ConditionalExpression:exit": scopes.pop,
      "LogicalExpression > .left:exit": scopes.push,
      "LogicalExpression:exit": scopes.pop,
      VariableDeclarator(node) {
        recordVariableTag(
          node,
          context,
          scopes.aliases,
          scopes.clearedForwardAliases,
          config,
          activeAliases(),
        );
      },
      AssignmentExpression(node) {
        recordAssignmentTag(
          node,
          context,
          scopes.aliases,
          scopes.clearedForwardAliases,
          config,
          activeAliases(),
        );
      },
      CallExpression: checkInvocation,
      NewExpression: checkInvocation,
      ExportNamedDeclaration(node) {
        checkExportLeaks(
          node,
          context,
          config,
          scopes.aliases,
          scopes.clearedForwardAliases,
          forwardAliases,
        );
      },
      ExportAllDeclaration(node) {
        checkExportLeaks(
          node,
          context,
          config,
          scopes.aliases,
          scopes.clearedForwardAliases,
          forwardAliases,
        );
      },
      ExportDefaultDeclaration(node) {
        checkDefaultExportLeak(
          node,
          context,
          config,
          scopes.aliases,
          scopes.clearedForwardAliases,
          forwardAliases,
        );
      },
    };
  },
);
