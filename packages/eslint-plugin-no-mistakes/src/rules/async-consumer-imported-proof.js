"use strict";
const { dirname } = require("node:path");
const { unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { propertiesWritten } = require("./async-disposition-mocks");
const { localFunction } = require("./async-consumer-local-proof");

// Oxlint's type-aware checker does not expose a TypeScript program to JS plugins.
// Resolve only the imported implementation; memoize sources/failures for this rule context.
function createImportedSources() {
  let ts, parser;
  try {
    ts = require("typescript");
    parser = require("@typescript-eslint/parser");
  } catch {
    // Optional tooling cannot establish a contract when unavailable or unloadable.
    return null;
  }
  // Legacy resolution may inspect `main` while runtime `exports` selects another body.
  if (ts.ModuleResolutionKind.Bundler === undefined) return null;
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
  return { resolve, load };
}

function exportedFunction(filename, name, sources, seen = new Set()) {
  if (seen.size >= 32 || seen.has(filename)) return null;
  seen.add(filename);
  const source = sources.load(filename);
  if (!source) return null;
  const matches = [];
  for (const statement of source.ast.body) {
    if (statement.type !== "ExportNamedDeclaration" || statement.exportKind === "type") continue;
    const declaration = statement.declaration;
    const ids =
      declaration?.type === "FunctionDeclaration"
        ? [declaration.id]
        : declaration?.type === "VariableDeclaration"
          ? declaration.declarations.map((item) => item.id)
          : [];
    for (const id of ids) if (id?.name === name) matches.push({ local: id });
    for (const item of statement.specifiers)
      if (item.exportKind !== "type" && (item.exported.name ?? item.exported.value) === name)
        matches.push({ local: item.local, source: statement.source });
  }
  if (matches.length !== 1) return null;
  const { local, source: reexport } = matches[0];
  if (reexport) {
    const target = sources.resolve(reexport.value, filename);
    return target ? exportedFunction(target, local.name, sources, seen) : null;
  }
  const fn = localFunction(local, source.context);
  const variable = resolveVariable(local, source.context);
  // Exporting preserves live binding identity; any other non-call use may mutate it.
  if (
    !fn ||
    variable.references.some((ref) => {
      if (ref.init) return false;
      const parent = ref.identifier.parent;
      return (
        parent.type !== "ExportSpecifier" &&
        !(parent.type === "CallExpression" && parent.callee === ref.identifier)
      );
    })
  )
    return null;
  return { fn, context: source.context };
}

function importedAwaited(call, index, context, state, property, proveBody) {
  const program = context.sourceCode.parserServices?.program;
  if (program?.getCompilerOptions().noResolve) return false;
  if (program && property === null)
    return require("./async-consumer-typed-proof").importedAwaited(
      call,
      index,
      context,
      state.typed,
    );

  const resolved = importedFunction(unwrapExpression(call.callee), context, state);
  if (!resolved) return false;
  const mode = proveBody(resolved.fn, index, resolved.context, property);
  return mode?.returned ? { ...mode, async: Boolean(resolved.fn.async) } : mode;
}

function importedFunction(callee, context, state) {
  const binding = resolveVariable(callee, context);
  if (
    binding?.defs.length !== 1 ||
    binding.defs[0].type !== "ImportBinding" ||
    binding.references.some((ref) => ref.isWrite()) ||
    propertiesWritten(callee, context, new Set(), null, true)
  )
    return false;
  const def = binding.defs[0];
  if (
    def.node.type !== "ImportSpecifier" ||
    def.parent.importKind === "type" ||
    def.node.importKind === "type"
  )
    return false;
  // Cache unavailable peers too; a new lint context retries without persisting state.
  if (state.sources === null) state.sources = createImportedSources() || false;
  const sources = state.sources;
  if (!sources) return false;
  const filename = context.filename;
  const target = sources.resolve(def.parent.source.value, filename);
  const resolved =
    target && exportedFunction(target, def.node.imported.name ?? def.node.imported.value, sources);
  return resolved;
}

module.exports = { importedAwaited, importedFunction };
