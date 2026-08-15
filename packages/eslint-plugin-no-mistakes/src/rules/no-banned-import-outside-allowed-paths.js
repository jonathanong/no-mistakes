"use strict";

const { rule } = require("../helpers");
const {
  hasBannedName,
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
const { tagForExpression } = require("./no-banned-import-outside-allowed-paths-tags");

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

    let aliases = new Map();
    const forwardAliases = new Map();
    let clearedForwardAliases = new Set();
    const aliasStack = [];
    const clearedAliasStack = [];
    let functionDepth = 0;

    function pushAliasScope() {
      aliasStack.push(aliases);
      clearedAliasStack.push(clearedForwardAliases);
      aliases = new Map(aliases);
      clearedForwardAliases = new Set(clearedForwardAliases);
    }

    function popAliasScope() {
      aliases = aliasStack.pop();
      clearedForwardAliases = clearedAliasStack.pop();
    }

    function isIifeFunction(node) {
      const parent = node?.parent;
      return parent?.type === "CallExpression" && parent.callee === node;
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

    // Merges forward-declared module-scope tags (imports, plus hoistable
    // require()/createRequire() forward references) with the current
    // block-scoped tags when resolving inside a function body. At module
    // depth 0, `aliases` alone reflects real top-to-bottom JS execution
    // order, so no forward merge happens there (matches the reference rule).
    function activeAliases() {
      if (functionDepth === 0) return aliases;
      const active = new Map(forwardAliases);
      for (const variable of clearedForwardAliases) active.delete(variable);
      for (const [variable, tag] of aliases) active.set(variable, tag);
      return active;
    }

    function reportCall(node, module, name) {
      context.report({ node, messageId: "bannedImport", data: { module, name } });
    }

    return {
      Program(node) {
        seedImportTags(node, context, config, aliases);
        for (const [variable, tag] of aliases) forwardAliases.set(variable, tag);
        collectBannedAliases(node, context, forwardAliases, config);
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
      VariableDeclarator(node) {
        recordVariableTag(node, context, aliases, clearedForwardAliases, config, activeAliases());
      },
      AssignmentExpression(node) {
        recordAssignmentTag(node, context, aliases, clearedForwardAliases, config, activeAliases());
      },
      CallExpression(node) {
        const tag = tagForExpression(node.callee, context, activeAliases(), config);
        if (tag?.kind === "direct") {
          reportCall(node.callee, tag.module, tag.name);
          return;
        }
        if (tag?.kind !== "object") return;
        for (const module of tag.modules) {
          if (hasBannedName(config, module, "default")) {
            reportCall(node.callee, module, "default");
            return;
          }
        }
      },
      ExportNamedDeclaration(node) {
        checkExportLeaks(node, context, config, aliases, forwardAliases);
      },
      ExportAllDeclaration(node) {
        checkExportLeaks(node, context, config, aliases, forwardAliases);
      },
      ExportDefaultDeclaration(node) {
        checkDefaultExportLeak(node, context, config, aliases, forwardAliases);
      },
    };
  },
);
