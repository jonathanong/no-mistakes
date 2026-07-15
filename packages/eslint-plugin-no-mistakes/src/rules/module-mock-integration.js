"use strict";

const { existsSync, readFileSync, statSync } = require("node:fs");
const { dirname, join, resolve } = require("node:path");
const { isInternalSpecifier, propertyName } = require("./module-mock-helpers");
const { analyzeFactory, spreadPreservesRealModule } = require("./module-mock-preserve-factory");

// Matches the repo's documented TS/JS source-extension set (docs/ast-analysis.md
// "Shared File Model"), so a barrel re-exporting a .tsx/.jsx leaf (e.g. a React
// component) resolves the same way the rest of the toolchain treats source files.
const DEFAULT_REEXPORT_EXTENSIONS = [".mts", ".ts", ".tsx", ".mjs", ".js", ".jsx", ".cts", ".cjs"];

function mockedExportNames(factory, specifier, mock, context) {
  if (!factory) return null;
  const analysis = analyzeFactory(factory, specifier, mock, context);
  const { objects } = analysis;
  if (objects.length === 0) return null;
  if (objects.some((object) => !object)) return null;
  const names = [];
  for (const object of objects) {
    for (const prop of object.properties) {
      if (prop.type === "SpreadElement") {
        if (!spreadPreservesRealModule(prop, analysis, specifier, mock, context)) return null;
        continue;
      }
      if (prop.computed) return null;
      const name = propertyName(prop.key);
      if (name && name !== "__esModule") names.push(name);
    }
  }
  return names.length > 0 ? names : null;
}

function integrationSourcePath(specifier, config) {
  const patterns = config.sourcePathTemplates ?? [];
  for (const template of patterns) {
    const prefix = config.specifierPrefix ?? "";
    const suffix = specifier.startsWith(prefix) ? specifier.slice(prefix.length) : specifier;
    for (const ext of config.extensions ?? [""]) {
      const candidate = template
        .replaceAll("{specifier}", specifier)
        .replaceAll("{specifierSuffix}", suffix)
        .replaceAll("{extension}", ext);
      const path = resolve(process.cwd(), candidate);
      if (existsSync(path)) return path;
    }
  }
  return null;
}

function safeRegExp(source, flags) {
  try {
    return new RegExp(source, flags);
  } catch {
    return null;
  }
}

// Built once per `integrationExportNames` call and reused across every file the
// local re-export graph reaches. Safe to share: `String.prototype.matchAll` clones
// the regex per call and never mutates the shared `lastIndex`.
function tagPatterns(config) {
  const marker = config.markerRegex ?? String.raw`/\*\s*no-mistakes:\s*integration=[^*]+\*/`;
  const declaration = safeRegExp(
    `${marker}\\s*export\\s+(?:async\\s+)?(?:function|const|let|var|class)\\s+([A-Za-z_$][\\w$]*)`,
    "g",
  );
  const defaultDeclaration = safeRegExp(
    `${marker}\\s*export\\s+default\\s+(?:async\\s+)?(?:function|class)\\b`,
    "g",
  );
  const named = safeRegExp(`${marker}\\s*export\\s*\\{([^}]+)\\}`, "g");
  if (!declaration || !defaultDeclaration || !named) return null;
  return { declaration, defaultDeclaration, named };
}

// `includeDefault` is false for every file reached via `export *`: ES modules never
// re-export a target's default binding through a star re-export, only through the
// root specifier itself or an explicit named re-export.
function addTaggedNames(source, patterns, names, includeDefault) {
  for (const match of source.matchAll(patterns.declaration)) names.add(match[1]);
  if (includeDefault) {
    for (const _match of source.matchAll(patterns.defaultDeclaration)) names.add("default");
  }
  for (const match of source.matchAll(patterns.named)) {
    for (const part of (match[1] ?? "").split(",")) {
      const exported = part
        .trim()
        .split(/\s+as\s+/)
        .pop()
        ?.trim();
      if (exported === "default" && !includeDefault) continue; // e.g. `export { x as default }`
      if (/^[A-Za-z_$][\w$]*$/.test(exported ?? "")) names.add(exported);
    }
  }
}

// Only plain `export * from '<specifier>'` re-exports propagate individual runtime
// export names; `export * as ns from ...` and type-only re-exports are intentionally
// left unmatched.
const REEXPORT_ALL = /export\s*\*\s*from\s*['"]([^'"]+)['"]/g;

// Matches whichever comes first at each position: a string/template literal (kept
// verbatim — it may be a real re-export's own specifier) or a line/block comment
// (dropped). This keeps a disabled `// export * from './leaf'` line, or the same
// text inside a `/* ... */` block, from being mistaken for a live barrel edge.
// A string literal whose *contents* merely spell out re-export-like text (rather
// than containing an actual disabled statement) is a narrower, accepted heuristic
// gap — matching the tag-marker scan's own text-based blind spots elsewhere in
// this file; resolving it would need a real lexer, not a regex pass.
const COMMENT_OR_STRING =
  /"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`|\/\/[^\n]*|\/\*[\s\S]*?\*\//g;

function withoutComments(source) {
  return source.replace(COMMENT_OR_STRING, (match) => (/^['"`]/.test(match) ? match : ""));
}

// NodeNext/ESM TypeScript projects conventionally write re-export specifiers with
// the compiled output extension (`./leaf.js`) even though the checked-in source is
// `./leaf.ts`; stripping a known compiled extension lets the configured `extensions`
// candidates resolve against the real source stem instead of probing `leaf.js.ts`.
const COMPILED_JS_EXTENSIONS = [".js", ".mjs", ".cjs"];

function resolveReexportPath(fromPath, specifier, extensions) {
  const base = resolve(dirname(fromPath), specifier);
  const compiledExt = COMPILED_JS_EXTENSIONS.find((ext) => specifier.endsWith(ext));
  const stem = compiledExt ? base.slice(0, -compiledExt.length) : null;
  const candidates = [
    base,
    ...(stem ? extensions.map((ext) => stem + ext) : []),
    ...extensions.map((ext) => base + ext),
    ...extensions.map((ext) => join(base, `index${ext}`)),
  ];
  for (const candidate of candidates) {
    if (existsSync(candidate) && statSync(candidate).isFile()) return candidate;
  }
  return null;
}

function reexportTargets(source, fromPath, extensions) {
  const targets = [];
  for (const match of withoutComments(source).matchAll(REEXPORT_ALL)) {
    const specifier = match[1];
    if (!specifier.startsWith(".")) continue; // leave bare-specifier re-exports unresolved
    const resolved = resolveReexportPath(fromPath, specifier, extensions);
    if (resolved) targets.push(resolved);
  }
  return targets;
}

function collectTaggedExports(path, extensions, patterns, names, visited, includeDefault) {
  if (visited.has(path)) return; // guard against re-export cycles
  visited.add(path);
  const source = readFileSync(path, "utf8");
  addTaggedNames(source, patterns, names, includeDefault);
  for (const target of reexportTargets(source, path, extensions)) {
    // `export *` never re-exports `default`, at any recursion depth.
    collectTaggedExports(target, extensions, patterns, names, visited, false);
  }
}

function integrationExportNames(specifier, config) {
  const path = integrationSourcePath(specifier, config);
  if (!path) return null;
  const patterns = tagPatterns(config);
  if (!patterns) return null;
  const extensions = config.reexportExtensions ?? DEFAULT_REEXPORT_EXTENSIONS;
  const names = new Set();
  collectTaggedExports(path, extensions, patterns, names, new Set(), true);
  return names;
}

function integrationAllows(specifier, factory, mock, context, options) {
  const config = options.integrationExports;
  if (!config) return false;
  const specifierPatterns = config.specifiers ?? [];
  if (
    specifierPatterns.length > 0 &&
    !isInternalSpecifier(specifier, { internalSpecifiers: specifierPatterns })
  ) {
    return false;
  }
  const mocked = mockedExportNames(factory, specifier, mock, context);
  if (!mocked) return false;
  const allowed = integrationExportNames(specifier, config);
  if (!allowed) return false;
  return mocked.every((name) => allowed.has(name));
}

module.exports = {
  integrationAllows,
};
