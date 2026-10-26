"use strict";

const { dirname } = require("node:path");
const { parsedSource } = require("./async-source-files");

function createSourceSession(ts, parser) {
  // Legacy resolution may inspect `main` while runtime `exports` selects another body.
  if (
    ts.ModuleResolutionKind.Bundler === undefined ||
    typeof ts.createModuleResolutionCache !== "function"
  )
    return null;
  const files = new Map();
  const configs = new Map();
  const configFiles = new Map();
  const resolved = new Map();
  const resolutionCaches = new Map();
  const host = { ...ts.sys };
  for (const method of ["readFile", "fileExists", "directoryExists", "realpath"]) {
    if (!ts.sys[method]) continue;
    const results = new Map();
    host[method] = (path) => {
      if (!results.has(path)) results.set(path, ts.sys[method](path));
      return results.get(path);
    };
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
  function options(filename) {
    const directory = dirname(filename);
    if (!configs.has(directory)) {
      const path = ts.findConfigFile(directory, host.fileExists);
      if (!configFiles.has(path)) {
        const config = path && ts.readConfigFile(path, host.readFile);
        configFiles.set(
          path,
          config?.error
            ? null
            : config
              ? compilerOptions(config.config, path)
              : { moduleResolution: ts.ModuleResolutionKind.Bundler },
        );
      }
      configs.set(directory, configFiles.get(path));
    }
    return configs.get(directory);
  }
  function resolve(source, filename) {
    if (!resolved.has(filename)) resolved.set(filename, new Map());
    const imports = resolved.get(filename);
    if (!imports.has(source)) {
      const config = options(filename);
      let target = null;
      if (config && !config.noResolve) {
        // Resolver caches must not mix paths/baseUrl from distinct tsconfigs.
        if (!resolutionCaches.has(config))
          resolutionCaches.set(
            config,
            ts.createModuleResolutionCache(process.cwd(), (path) => path, config),
          );
        target = ts.resolveModuleName(source, filename, config, host, resolutionCaches.get(config))
          .resolvedModule?.resolvedFileName;
      }
      imports.set(source, target);
    }
    return imports.get(source);
  }
  function load(filename) {
    if (!files.has(filename)) {
      files.set(filename, null);
      const text = host.readFile(filename);
      try {
        files.set(filename, parsedSource(filename, text, parser));
      } catch {
        // Missing, ambient, or unsupported source never establishes an awaiting contract.
      }
    }
    return files.get(filename);
  }
  return { resolve, load };
}

module.exports = { createSourceSession };
