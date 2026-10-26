"use strict";
const { createImportedSources } = require("./async-source-store");
const { unwrapExpression } = require("./async-ast");
const { resolveVariable } = require("./async-target-bindings");
const { propertiesWritten } = require("./async-disposition-mocks");
const { localFunction } = require("./async-consumer-local-proof");

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
  if (state.sources === null) state.sources = createImportedSources(context) || false;
  const sources = state.sources;
  if (!sources) return false;
  const filename = context.filename;
  const target = sources.resolve(def.parent.source.value, filename);
  const resolved =
    target && exportedFunction(target, def.node.imported.name ?? def.node.imported.value, sources);
  return resolved;
}

module.exports = { importedAwaited, importedFunction, createImportedSources };
