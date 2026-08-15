"use strict";

const { importSpecifierName } = require("./module-mock-helpers");
const { resolveVariable } = require("./no-global-fetch-outside-helper-bindings");
const {
  hasAnyBannedName,
  hasBannedName,
} = require("./no-banned-import-outside-allowed-paths-config");

const CREATE_REQUIRE_MODULES = new Set(["node:module", "module"]);

function isTypeOnlyImport(node, specifier) {
  return node.importKind === "type" || specifier.importKind === "type";
}

function isTypeOnlyExport(node, specifier) {
  return node.exportKind === "type" || specifier.exportKind === "type";
}

function seedImportSpecifier(specifier, node, moduleSpecifier, context, config, aliasMap) {
  if (isTypeOnlyImport(node, specifier)) return;
  const variable = resolveVariable(specifier.local, context);
  if (!variable) return;
  if (specifier.type === "ImportSpecifier") {
    const name = importSpecifierName(specifier);
    if (CREATE_REQUIRE_MODULES.has(moduleSpecifier) && name === "createRequire") {
      aliasMap.set(variable, { kind: "create-require" });
      return;
    }
    if (name && hasBannedName(config, moduleSpecifier, name)) {
      aliasMap.set(variable, { kind: "direct", module: moduleSpecifier, name });
    }
    return;
  }
  if (
    specifier.type === "ImportDefaultSpecifier" ||
    specifier.type === "ImportNamespaceSpecifier"
  ) {
    if (hasAnyBannedName(config, moduleSpecifier)) {
      aliasMap.set(variable, { kind: "object", modules: new Set([moduleSpecifier]) });
    }
  }
}

// Seeds import-derived tags. Import bindings are always top-level and
// unconditional (re-assigning them is a syntax error), so this is a single
// non-fixed-point pass, unlike the require()/createRequire() forward pass.
function seedImportTags(program, context, config, aliasMap) {
  for (const node of program.body) {
    if (node.type !== "ImportDeclaration") continue;
    const moduleSpecifier = node.source.value;
    for (const specifier of node.specifiers) {
      seedImportSpecifier(specifier, node, moduleSpecifier, context, config, aliasMap);
    }
  }
}

function specifierSourceName(node) {
  return node.type === "Literal" ? String(node.value) : node.name;
}

function reportTagLeak(reportNode, tag, config, context) {
  if (tag?.kind === "direct") {
    context.report({
      node: reportNode,
      messageId: "bannedReExport",
      data: { module: tag.module, name: tag.name },
    });
    return true;
  }
  if (tag?.kind !== "object") return false;
  for (const module of tag.modules) {
    if (hasAnyBannedName(config, module)) {
      context.report({
        node: reportNode,
        messageId: "bannedReExport",
        data: { module, name: "*" },
      });
      return true;
    }
  }
  return false;
}

function reportLocalReExport(specifier, context, config, aliasMap, forwardAliasMap) {
  const variable = resolveVariable(specifier.local, context);
  const tag = variable && (aliasMap.get(variable) ?? forwardAliasMap.get(variable));
  reportTagLeak(specifier, tag, config, context);
}

function checkExportLeaks(node, context, config, aliasMap, forwardAliasMap) {
  if (node.type === "ExportAllDeclaration") {
    const moduleSpecifier = node.source?.value;
    if (hasAnyBannedName(config, moduleSpecifier)) {
      context.report({
        node,
        messageId: "bannedReExport",
        data: { module: moduleSpecifier, name: "*" },
      });
    }
    return;
  }
  for (const specifier of node.specifiers ?? []) {
    if (specifier.type !== "ExportSpecifier" || isTypeOnlyExport(node, specifier)) continue;
    if (node.source) {
      const sourceName = specifierSourceName(specifier.local);
      if (hasBannedName(config, node.source.value, sourceName)) {
        context.report({
          node: specifier,
          messageId: "bannedReExport",
          data: { module: node.source.value, name: sourceName },
        });
      }
      continue;
    }
    reportLocalReExport(specifier, context, config, aliasMap, forwardAliasMap);
  }
}

function checkDefaultExportLeak(node, context, config, aliasMap, forwardAliasMap) {
  if (node.declaration.type !== "Identifier") return;
  const variable = resolveVariable(node.declaration, context);
  const tag = variable && (aliasMap.get(variable) ?? forwardAliasMap.get(variable));
  reportTagLeak(node, tag, config, context);
}

module.exports = {
  checkDefaultExportLeak,
  checkExportLeaks,
  seedImportTags,
};
