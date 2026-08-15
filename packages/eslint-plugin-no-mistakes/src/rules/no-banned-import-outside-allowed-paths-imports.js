"use strict";

const { importSpecifierName } = require("./module-mock-helpers");
const { resolveVariable } = require("./no-global-fetch-outside-helper-bindings");
const {
  CREATE_REQUIRE_MODULES,
  hasAnyBannedName,
  hasAnyNonDefaultBannedName,
  hasBannedName,
} = require("./no-banned-import-outside-allowed-paths-config");
const {
  resolveNodeModuleCreateRequireTag,
  tagForExpression,
} = require("./no-banned-import-outside-allowed-paths-tags");

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
    const createRequireTag = resolveNodeModuleCreateRequireTag(moduleSpecifier, name);
    if (createRequireTag) {
      aliasMap.set(variable, createRequireTag);
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
    // A CREATE_REQUIRE_MODULES namespace/default binding is tracked as an
    // object whether or not the config separately bans anything from it, so
    // a member access like `mod.createRequire` still resolves below (see
    // resolveNodeModuleCreateRequireTag) even when nothing else about the
    // module is configured as banned.
    if (hasAnyBannedName(config, moduleSpecifier) || CREATE_REQUIRE_MODULES.has(moduleSpecifier)) {
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

// Resolves the tag reachable through `variable` for an export check: prefer
// a live real-time tag, but never fall back to the fixed-point forward tag
// once the variable has been explicitly cleared (real-time overwritten to
// something untracked) — otherwise a since-overwritten alias would "revive"
// its stale, no-longer-true forward tag and produce a false positive. A
// forward tag is only consulted for a variable never yet touched in real
// time, i.e. a genuine forward reference.
function resolveExportedTag(variable, aliasMap, clearedAliases, forwardAliasMap) {
  if (!variable) return null;
  if (aliasMap.has(variable)) return aliasMap.get(variable);
  if (clearedAliases.has(variable)) return null;
  return forwardAliasMap.get(variable) ?? null;
}

function reportLocalReExport(
  specifier,
  context,
  config,
  aliasMap,
  clearedAliases,
  forwardAliasMap,
) {
  const variable = resolveVariable(specifier.local, context);
  const tag = resolveExportedTag(variable, aliasMap, clearedAliases, forwardAliasMap);
  reportTagLeak(specifier, tag, config, context);
}

function checkExportedDeclaration(
  declaration,
  context,
  config,
  aliasMap,
  clearedAliases,
  forwardAliasMap,
) {
  if (declaration?.type !== "VariableDeclaration") return;
  for (const declarator of declaration.declarations) {
    if (declarator.id.type !== "Identifier") continue;
    const variable = resolveVariable(declarator.id, context);
    const tag = resolveExportedTag(variable, aliasMap, clearedAliases, forwardAliasMap);
    reportTagLeak(declarator, tag, config, context);
  }
}

function checkExportLeaks(node, context, config, aliasMap, clearedAliases, forwardAliasMap) {
  if (node.type === "ExportAllDeclaration") {
    const moduleSpecifier = node.source?.value;
    // An unaliased `export * from "mod"` never re-exports the module's
    // default export (ES module semantics), so a module banned only on
    // "default" exposes nothing reachable through this form.
    if (hasAnyNonDefaultBannedName(config, moduleSpecifier)) {
      context.report({
        node,
        messageId: "bannedReExport",
        data: { module: moduleSpecifier, name: "*" },
      });
    }
    return;
  }
  // An inline export declaration (`export const compile = ts.createProgram;`)
  // exposes a tagged value directly, with no `specifiers` entry to inspect.
  checkExportedDeclaration(
    node.declaration,
    context,
    config,
    aliasMap,
    clearedAliases,
    forwardAliasMap,
  );
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
    reportLocalReExport(specifier, context, config, aliasMap, clearedAliases, forwardAliasMap);
  }
}

function checkDefaultExportLeak(node, context, config, aliasMap, clearedAliases, forwardAliasMap) {
  const { declaration } = node;
  if (declaration.type === "Identifier") {
    const variable = resolveVariable(declaration, context);
    const tag = resolveExportedTag(variable, aliasMap, clearedAliases, forwardAliasMap);
    reportTagLeak(node, tag, config, context);
    return;
  }
  // A non-identifier declaration (`export default ts.createProgram;` or
  // `export default require("typescript").createProgram;`) is resolved with
  // the same expression tagger used for calls and aliases, matching
  // real-time (depth-0, no forward merge) semantics.
  const tag = tagForExpression(declaration, context, aliasMap, config);
  reportTagLeak(node, tag, config, context);
}

module.exports = {
  checkDefaultExportLeak,
  checkExportLeaks,
  seedImportTags,
};
