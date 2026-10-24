"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { childNodes, unwrapTs } = require("./postgres-query-text");
const { parameterExecutors } = require("./postgres-scoped-parameters");
const { isScopedExecutor, isScopedMember } = require("./postgres-scoped-lookup");

const RELATIVE_EXTENSIONS = [".ts", ".tsx", ".mts", ".cts", ".js", ".mjs", ".cjs", ".jsx"];

const FUNCTION_TYPES = new Set([
  "FunctionDeclaration",
  "FunctionExpression",
  "ArrowFunctionExpression",
]);
const SCOPE_TYPES = new Set([
  "Program",
  "BlockStatement",
  "StaticBlock",
  "SwitchStatement",
  "ForStatement",
]);

function importedName(specifier) {
  const imported = specifier?.imported;
  return imported?.type === "Literal" ? String(imported.value) : imported?.name;
}

// The module itself or any subpath of it; `@example/dbx` does not match.
function fromConfiguredModule(source, specifier) {
  return !specifier || source === specifier || String(source).startsWith(`${specifier}/`);
}

function packageName(specifier) {
  if (
    typeof specifier !== "string" ||
    specifier === "" ||
    specifier.startsWith(".") ||
    specifier.startsWith("/")
  ) {
    return null;
  }
  if (specifier.startsWith("@")) {
    const slash = specifier.indexOf("/");
    if (slash <= 1) return null;
    const rest = specifier.slice(slash + 1);
    const name = rest.split("/")[0];
    if (!name) return null;
    return specifier.slice(0, slash + 1 + name.length);
  }
  const name = specifier.split("/")[0];
  return name || null;
}

function isRelativeSpecifier(source) {
  return (
    source === "." ||
    source === ".." ||
    (typeof source === "string" && (source.startsWith("./") || source.startsWith("../")))
  );
}

function readPackageName(dir) {
  try {
    const parsed = JSON.parse(fs.readFileSync(path.join(dir, "package.json"), "utf8"));
    return typeof parsed.name === "string" ? parsed.name : null;
  } catch {
    return null;
  }
}

// Walk up from the linted file. A missing or unreadable root fails closed.
function packageRoot(filename, specifier) {
  const name = packageName(specifier);
  if (!name || typeof filename !== "string" || filename === "") return null;
  let dir = path.dirname(path.resolve(filename));
  const stop = path.parse(dir).root;
  while (dir) {
    if (readPackageName(dir) === name) return dir;
    const nested = path.join(dir, "node_modules", ...name.split("/"));
    if (readPackageName(nested) === name) return nested;
    if (dir === stop) return null;
    const parent = path.dirname(dir);
    if (parent === dir) return null;
    dir = parent;
  }
  return null;
}

function resolveRelativeFile(filename, specifier) {
  if (typeof filename !== "string" || typeof specifier !== "string") return null;
  const base = path.resolve(path.dirname(path.resolve(filename)), specifier);
  const candidates = [base];
  for (const extension of RELATIVE_EXTENSIONS) candidates.push(`${base}${extension}`);
  for (const extension of RELATIVE_EXTENSIONS) {
    candidates.push(path.join(base, `index${extension}`));
  }
  for (const candidate of candidates) {
    try {
      if (fs.statSync(candidate).isFile()) return candidate;
    } catch {
      // Missing paths and unreadable entries fail closed.
    }
  }
  return null;
}

function fileInsidePackage(file, root) {
  const relative = path.relative(root, file);
  return relative !== "" && !relative.startsWith("..") && !path.isAbsolute(relative);
}

function matchesConfiguredModule(source, specifier, filename) {
  if (fromConfiguredModule(source, specifier)) return true;
  if (!specifier || !isRelativeSpecifier(source)) return false;
  const root = packageRoot(filename, specifier);
  if (!root) return false;
  const resolved = resolveRelativeFile(filename, source);
  return resolved !== null && fileInsidePackage(resolved, root);
}

// Local names of configured factory (value) and type imports, by module.
function scopedImports(program, options, filename) {
  const factories = new Set();
  const types = new Set();
  const { importSpecifier, executorFactoryNames, executorTypeNames } = options;
  for (const statement of program?.body ?? []) {
    if (statement.type !== "ImportDeclaration") continue;
    if (!matchesConfiguredModule(statement.source?.value, importSpecifier, filename)) continue;
    for (const specifier of statement.specifiers ?? []) {
      if (specifier.type !== "ImportSpecifier") continue;
      const imported = importedName(specifier);
      const typeOnly = statement.importKind === "type" || specifier.importKind === "type";
      if (!typeOnly && executorFactoryNames.includes(imported)) factories.add(specifier.local.name);
      if (executorTypeNames.includes(imported)) types.add(specifier.local.name);
    }
  }
  return { factories, types };
}

function isFactoryCall(init, factories) {
  let node = unwrapTs(init);
  if (node?.type === "AwaitExpression") node = unwrapTs(node.argument);
  const callee = node?.type === "CallExpression" ? unwrapTs(node.callee) : null;
  return callee?.type === "Identifier" && factories.has(callee.name);
}

/**
 * Executors that exist only inside a lexical scope: locals bound to a
 * configured factory call (the declaring block) and parameters typed with a
 * configured type (the declaring function). Property entries also carry the
 * receiver's declared member name; the receiver itself is not an executor.
 */
function collectScopedExecutors(program, options, filename, context) {
  const found = [];
  const { factories, types } = scopedImports(program, options, filename);
  if (
    factories.size === 0 &&
    types.size === 0 &&
    (!context?.sourceCode?.parserServices?.program || options.executorTypeNames.length === 0)
  )
    return found;
  const walk = (node, scope) => {
    if (SCOPE_TYPES.has(node.type)) scope = node;
    if (node.type === "VariableDeclaration" && node.kind !== "var") {
      for (const declarator of node.declarations) {
        if (declarator.id.type !== "Identifier" || !isFactoryCall(declarator.init, factories)) {
          continue;
        }
        found.push({
          name: declarator.id.name,
          range: [declarator.range[0], scope.range[1]],
        });
      }
    }
    if (FUNCTION_TYPES.has(node.type)) {
      for (const param of node.params) {
        const entries = parameterExecutors(param, types, context, options);
        for (const name of entries.names) {
          found.push({ name, range: node.range });
        }
        for (const member of entries.members) found.push({ ...member, range: node.range });
      }
    }
    for (const child of childNodes(node)) walk(child, scope);
  };
  walk(program, program);
  return found;
}

module.exports = {
  collectScopedExecutors,
  isScopedExecutor,
  isScopedMember,
  matchesConfiguredModule,
};
