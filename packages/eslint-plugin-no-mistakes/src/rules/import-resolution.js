"use strict";

const { unwrapSequence } = require("./async-ast");
const { memberFact, rootIdentifier } = require("./import-member");
const { hasImportWrite, laterFact } = require("./import-later-fact");
const {
  factAtPath,
  namespaceMember,
  patternDefault,
  patternPath,
  withPatternDefault,
} = require("./import-pattern-path");
const { literalString } = require("./logical-call");
const { resolveRuntimeVariable, resolveVariable } = require("./async-target-bindings");

function reportsUnanalyzable(context) {
  return (context.options?.[0]?.unanalyzable ?? "finding") === "finding";
}

function isFreeRequire(node, context) {
  const current = unwrapSequence(node);
  if (current?.type !== "Identifier" || current.name !== "require") return false;
  if (!context?.sourceCode?.getScope) return true;
  const variable = resolveRuntimeVariable(current, context);
  if (!variable || variable.defs.length === 0) return true;
  return !variable.defs.some(
    (def) =>
      def.type &&
      !["ImplicitGlobalVariable", "Type", "TypeParameter"].includes(def.type) &&
      !typeOnlyImport(def),
  );
}

function requiredMember(callee, context, seen = new Set(), memo) {
  return memberFact(callee, context, importFact, requireModule, namespaceMember, seen, memo);
}

// undefined: not a require. null: require whose module is not a static string.
function requireModule(node, context) {
  const call = unwrapSequence(node);
  if (call?.type !== "CallExpression") return undefined;
  if (!isFreeRequire(call.callee, context) || call.arguments.length !== 1) {
    return isFreeRequire(call.callee, context) ? null : undefined;
  }
  return literalString(call.arguments[0]);
}

function typeOnlyImport(def) {
  return def?.node?.importKind === "type" || def?.parent?.importKind === "type";
}

function importedSpecifierName(specifier) {
  const imported = specifier?.imported;
  if (imported?.type === "Identifier") return imported.name;
  return imported?.type === "Literal" ? String(imported.value) : null;
}

function importBindingFact(def) {
  const node = def.node;
  if (node?.type === "TSImportEqualsDeclaration") {
    const reference = node.moduleReference;
    if (reference?.type !== "TSExternalModuleReference") return { unresolved: true };
    const moduleName = literalString(reference.expression);
    if (moduleName === null) return { unresolved: true };
    return { module: moduleName, kind: "namespace", imported: null, typeOnly: typeOnlyImport(def) };
  }
  const moduleName = def.parent?.source?.value;
  if (typeof moduleName !== "string") return { unresolved: true };
  const typeOnly = typeOnlyImport(def);
  if (node?.type === "ImportNamespaceSpecifier" || node?.type === "ImportDefaultSpecifier")
    return {
      module: moduleName,
      kind: node.type === "ImportDefaultSpecifier" ? "default" : "namespace",
      imported: null,
      typeOnly,
    };
  if (node?.type === "ImportSpecifier") {
    const imported = importedSpecifierName(node);
    if (!imported) return { unresolved: true };
    if (imported === "default")
      return { module: moduleName, kind: "default", imported: null, typeOnly };
    return { module: moduleName, kind: "named", imported, typeOnly };
  }
  return { unresolved: true };
}

function destructureFact(declarator, name, context, seen) {
  const imported = patternPath(declarator.id, name, literalString, [], context);
  if (imported === null) return null;
  const required = requireModule(declarator.init, context);
  let fact;
  if (required === undefined) {
    const source = importFact(declarator.init, context, new Set(seen));
    if (!source) fact = null;
    else if (imported === undefined || source.unresolved || source.typeOnly)
      fact = source.typeOnly ? null : { unresolved: true };
    else fact = factAtPath(source, imported, namespaceMember);
  } else if (imported === undefined || required === null) fact = { unresolved: true };
  else
    fact = factAtPath(
      { module: required, kind: "namespace", typeOnly: false },
      imported,
      namespaceMember,
    );
  const fallback = patternDefault(declarator.id, name, context);
  if (!fallback) return fact;
  return withPatternDefault(fact, importFact(fallback, context, new Set(seen)));
}

function factFromDef(def, name, context, seen) {
  if (def?.type === "ImportBinding") return importBindingFact(def);
  if (def?.type !== "Variable" || def.node?.id?.type !== "ObjectPattern") return null;
  return destructureFact(def.node, name, context, seen);
}

function defMayBeImport(def, context, seen) {
  if (def?.type === "ImportBinding") return true;
  if (def?.type !== "Variable" || !def.node?.init) return false;
  let init = unwrapSequence(def.node.init);
  while (init?.type === "MemberExpression") init = unwrapSequence(init.object);
  if (!init) return false;
  if (requireModule(init, context) !== undefined) return true;
  if (init.type !== "Identifier" || seen.has(init.name)) return false;
  seen.add(init.name);
  const variable = resolveVariable(init, context);
  return Boolean(variable?.defs.some((item) => defMayBeImport(item, context, seen)));
}

function reassignedFact(def, variable, context, seen, memo) {
  const fact = (node) => node && importFact(node, context, new Set(seen), memo);
  if (fact(def.node.init)) return { unresolved: true };
  for (const reference of variable.references ?? []) {
    if (!reference.isWrite() || reference.init) continue;
    const parent = reference.identifier?.parent;
    const value =
      parent?.type === "AssignmentExpression" &&
      parent.left === reference.identifier &&
      parent.right;
    if (!value || fact(value)) return { unresolved: true };
  }
  return null;
}

function importFact(node, context, seen = new Set(), memo = new Map()) {
  const current = unwrapSequence(node);
  if (!current || !context?.sourceCode?.getScope) return null;
  const required = requireModule(current, context);
  if (required === null) return { unresolved: true };
  if (typeof required === "string")
    return { module: required, kind: "namespace", imported: null, typeOnly: false };
  const member = requiredMember(current, context, seen, memo);
  if (member) return member.unresolved ? { unresolved: true } : member;
  if (current.type !== "Identifier") return null;
  const variable = resolveRuntimeVariable(current, context) ?? resolveVariable(current, context);
  if (!variable || variable.defs.length === 0) return null;
  const cacheKey = current.range ? current.range[0] : variable;
  if (memo.has(cacheKey)) return memo.get(cacheKey);
  if (seen.has(variable)) return { unresolved: true };
  seen.add(variable);
  const result = readBinding(variable, current, context, seen, memo);
  memo.set(cacheKey, result);
  return result;
}

function readBinding(variable, current, context, seen, memo) {
  if (
    variable.defs.length !== 1 &&
    !variable.defs.some((def) => defMayBeImport(def, context, new Set([current.name]))) &&
    !hasImportWrite(variable, context, seen, memo, importFact)
  )
    return null;
  if (variable.defs.length !== 1) return { unresolved: true };
  const def = variable.defs[0];
  const direct = factFromDef(def, current.name, context, seen);
  const written = (variable.references ?? []).some(
    (reference) => reference.isWrite() && !reference.init,
  );
  if (direct) return written && !direct.unresolved ? { unresolved: true } : direct;
  const init = def.type === "Variable" && def.node?.id?.type === "Identifier" && def.node.init;
  if (!init) return laterFact(def, variable, context, seen, memo, importFact, current);
  if (def.parent?.kind !== "const" && written)
    return reassignedFact(def, variable, context, seen, memo);
  return importFact(init, context, seen, memo);
}

function importSpellings(root, localPath, context) {
  if (!root || !localPath) return { names: [], unresolved: false };
  const fact = importFact(root, context);
  if (!fact) return { names: [], unresolved: false };
  if (fact.unresolved) return { names: [], unresolved: true };
  if (fact.typeOnly) return { names: [], unresolved: false };
  if (fact.kind === "named" && fact.imported && (fact.name ?? fact.imported) !== root.name) {
    const renamed = `${fact.name || fact.imported}${localPath.slice(root.name.length)}`;
    if (localPath === root.name) return { names: [fact.name || fact.imported], unresolved: false };
    if (localPath.startsWith(`${root.name}.`)) return { names: [renamed], unresolved: false };
  }
  if (fact.kind === "namespace" && localPath.startsWith(`${root.name}.`))
    return { names: [localPath.slice(root.name.length + 1)], unresolved: false };
  return { names: [], unresolved: false };
}

module.exports = {
  importFact,
  importSpellings,
  reportsUnanalyzable,
  requireModule,
  requiredMember,
  rootIdentifier,
  typeOnlyImport,
};
