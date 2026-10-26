"use strict";

const { resolveVariable, stableConstInitializer } = require("./async-target-bindings");
const { unwrapExpression } = require("./async-ast");

// Follow named declarations only. Missing source, cycles, and changing bindings
// cannot establish the identity of a configured target or dependency type.
function createDeclarations(sources) {
  function exported(filename, name, seen = new Set()) {
    const key = `${filename}:${name}`;
    if (seen.size >= 32 || seen.has(key)) return null;
    seen = new Set(seen).add(key);
    const source = sources.load(filename);
    if (!source) return null;
    const matches = [];
    for (const statement of source.ast.body) {
      if (statement.type !== "ExportNamedDeclaration") continue;
      const declaration = statement.declaration;
      if (declaration?.id?.name === name) matches.push({ local: declaration.id });
      if (declaration?.type === "VariableDeclaration")
        for (const item of declaration.declarations)
          if (item.id.type === "Identifier" && item.id.name === name)
            matches.push({ local: item.id });
      for (const item of statement.specifiers)
        if ((item.exported.name ?? item.exported.value) === name)
          matches.push({ local: item.local, source: statement.source });
    }
    if (matches.length !== 1) return null;
    const match = matches[0];
    if (match.source) {
      const target = sources.resolve(match.source.value, filename);
      return target && exported(target, match.local.name ?? match.local.value, seen);
    }
    return declaration(match.local, source.context, seen);
  }

  function imported(id, context) {
    // Both callers validate the identifier before resolving this import.
    const variable = resolveVariable(id, context);
    if (
      variable?.defs.length !== 1 ||
      variable.references.some((ref) => ref.isWrite() && !ref.init)
    )
      return null;
    const def = variable.defs[0];
    return def.type === "ImportBinding" && def.node.type === "ImportSpecifier"
      ? { source: def.parent.source.value, name: def.node.imported.name ?? def.node.imported.value }
      : null;
  }

  function declaration(id, context, seen = new Set()) {
    if (id?.type !== "Identifier" || seen.size >= 32) return null;
    const variable = resolveVariable(id, context);
    if (!variable || seen.has(variable) || variable.defs.length !== 1) return null;
    if (variable.references.some((ref) => ref.isWrite() && !ref.init)) return null;
    const def = variable.defs[0];
    const importInfo = imported(id, context);
    if (importInfo) {
      const target = sources.resolve(importInfo.source, context.filename);
      return target && exported(target, importInfo.name, new Set(seen).add(variable));
    }
    return { node: def.node, context, id, kind: def.type };
  }

  function callable(id, context, seen = new Set()) {
    const resolved = declaration(id, context, seen);
    if (!resolved || seen.has(resolved.node) || seen.size >= 32) return null;
    const { node, context: owner } = resolved;
    // A parameter definition also points at its enclosing function, not a callee implementation.
    if (["FunctionDeclaration", "TSDeclareFunction"].includes(node.type))
      return resolved.kind === "FunctionName" ? resolved : null;
    if (node.type !== "VariableDeclarator") return null;
    const init = unwrapExpression(stableConstInitializer(resolved.id, owner, new Set()));
    if (["ArrowFunctionExpression", "FunctionExpression"].includes(init?.type))
      return { ...resolved, node: init };
    return callable(init, owner, new Set(seen).add(node));
  }

  return { declaration, imported, callable };
}

module.exports = { createDeclarations };
