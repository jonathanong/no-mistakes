"use strict";
const { dirname } = require("node:path");

// Oxlint's type-aware checker does not expose a TypeScript program to JS plugins.
// Resolve only the imported implementation; memoize sources/failures for this rule context.
function createImportedSources(context) {
  const ts = require("typescript");
  const parser = require("@typescript-eslint/parser");
  const reads = new Map();
  const files = new Map();
  const configs = new Map();
  const host = {
    ...ts.sys,
    readFile(path) {
      if (!reads.has(path)) reads.set(path, ts.sys.readFile(path));
      return reads.get(path);
    },
  };
  const resolutions = ts.createModuleResolutionCache(process.cwd(), (path) => path);
  function options(filename) {
    const directory = dirname(filename);
    if (!configs.has(directory)) {
      const path = ts.findConfigFile(directory, host.fileExists);
      const config = path && ts.readConfigFile(path, host.readFile);
      configs.set(
        directory,
        config?.error
          ? null
          : config
            ? compilerOptions(config.config, path)
            : { moduleResolution: ts.ModuleResolutionKind.Bundler },
      );
    }
    return configs.get(directory);
  }
  function compilerOptions(config, path) {
    const parsed = ts.parseJsonConfigFileContent(
      config,
      { ...host, readDirectory: () => [] },
      dirname(path),
      undefined,
      path,
    );
    return parsed.errors.some((error) => error.code !== 18003) ? null : parsed.options;
  }
  function resolve(source, filename) {
    const config = options(filename);
    if (!config || config.noResolve) return null;
    return ts.resolveModuleName(source, filename, config, host, resolutions).resolvedModule
      ?.resolvedFileName;
  }
  function load(filename) {
    if (!files.has(filename)) {
      files.set(filename, null);
      const text = host.readFile(filename);
      try {
        const parsed = parser.parseForESLint(text, { filePath: filename, range: true, loc: true });
        function parents(node, parent) {
          node.parent = parent;
          for (const key of parsed.visitorKeys[node.type] || []) {
            const child = node[key];
            if (Array.isArray(child)) child.forEach((item) => item && parents(item, node));
            else if (child) parents(child, node);
          }
        }
        parents(parsed.ast, null);
        const sourceCode = {
          ...parsed,
          getScope(node) {
            // Every parsed node has a Program ancestor with a global scope.
            for (let owner = node; ; owner = owner.parent) {
              const scope = parsed.scopeManager.acquire(owner, true);
              if (scope) return scope;
            }
          },
        };
        files.set(filename, { ast: parsed.ast, context: { sourceCode, filename } });
      } catch {
        // Missing, ambient, or unsupported source never establishes an awaiting contract.
      }
    }
    return files.get(filename);
  }
  if (context) files.set(context.filename, { ast: context.sourceCode.ast, context });
  return { resolve, load };
}

module.exports = { createImportedSources };
