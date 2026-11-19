"use strict";

// One syntactic pass over import/export declarations. It keeps only the
// specifier name nodes, keyed by the source name text, so the escape proof can
// follow renames and re-exports without resolving every identifier.
const indexes = new WeakMap();

function recordStatement(statement, ts, add, namespaces) {
  if (ts.isModuleDeclaration(statement)) {
    if (statement.body && ts.isModuleBlock(statement.body))
      for (const inner of statement.body.statements) recordStatement(inner, ts, add, namespaces);
  } else if (ts.isImportEqualsDeclaration(statement)) {
    const reference = statement.moduleReference;
    if (ts.isExternalModuleReference(reference)) namespaces.push(statement.name);
    else add(ts.isIdentifier(reference) ? reference.text : reference.right.text, statement.name);
  } else if (ts.isImportDeclaration(statement) || ts.isExportDeclaration(statement)) {
    const clause = ts.isImportDeclaration(statement)
      ? statement.importClause?.namedBindings
      : statement.exportClause;
    if (statement.importClause?.name) add("default", statement.importClause.name);
    if (clause && (ts.isNamespaceImport(clause) || ts.isNamespaceExport(clause)))
      namespaces.push(clause.name);
    for (const specifier of clause?.elements ?? [])
      add((specifier.propertyName ?? specifier.name).text, specifier.name);
  }
}

function programFiles(program) {
  return program
    .getSourceFiles()
    .filter(
      (file) =>
        !program.isSourceFileFromExternalLibrary(file) && !program.isSourceFileDefaultLibrary(file),
    );
}

function buildIndex(program, ts) {
  const bySource = new Map();
  const namespaces = [];
  const add = (key, name) => {
    const list = bySource.get(key);
    if (list) list.push(name);
    else bySource.set(key, [name]);
  };
  for (const file of programFiles(program))
    for (const statement of file.statements) recordStatement(statement, ts, add, namespaces);
  return { bySource, namespaces, modules: new Map() };
}

function indexFor(program, ts) {
  if (!indexes.has(program)) indexes.set(program, buildIndex(program, ts));
  return indexes.get(program);
}

module.exports = { indexFor, programFiles };
