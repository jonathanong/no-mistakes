"use strict";

const fs = require("node:fs");
const path = require("node:path");
const { childNodes, unwrapTs } = require("./postgres-query-text");

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
  if (typeof specifier !== "string" || specifier === "" || specifier.startsWith(".") || specifier.startsWith("/")) {
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

function typeMatches(annotation, types) {
  const node = annotation?.typeAnnotation ?? annotation;
  if (node?.type === "TSTypeReference") {
    return node.typeName?.type === "Identifier" && types.has(node.typeName.name);
  }
  if (node?.type === "TSUnionType") return node.types.some((member) => typeMatches(member, types));
  return false;
}

function propertyKeyName(key) {
  if (key?.type === "Identifier") return key.name;
  return key?.type === "Literal" ? String(key.value) : null;
}

function isFactoryCall(init, factories) {
  let node = unwrapTs(init);
  if (node?.type === "AwaitExpression") node = unwrapTs(node.argument);
  const callee = node?.type === "CallExpression" ? unwrapTs(node.callee) : null;
  return callee?.type === "Identifier" && factories.has(callee.name);
}

function parameterNames(param, types) {
  const target = param.type === "AssignmentPattern" ? param.left : param;
  if (target.type === "Identifier") {
    return typeMatches(target.typeAnnotation, types) ? [target.name] : [];
  }
  const literal = target.typeAnnotation?.typeAnnotation;
  if (target.type !== "ObjectPattern" || literal?.type !== "TSTypeLiteral") return [];
  const names = [];
  for (const property of target.properties) {
    const value =
      property.value?.type === "AssignmentPattern" ? property.value.left : property.value;
    if (property.type !== "Property" || value?.type !== "Identifier") continue;
    const key = propertyKeyName(property.key);
    const typed = literal.members.some(
      (member) =>
        member.type === "TSPropertySignature" &&
        propertyKeyName(member.key) === key &&
        typeMatches(member.typeAnnotation, types),
    );
    if (typed) names.push(value.name);
  }
  return names;
}

/**
 * Executors that exist only inside a lexical scope: locals bound to a
 * configured factory call (the declaring block) and parameters typed with a
 * configured type (the declaring function). Returns `{ name, range }` entries.
 */
function collectScopedExecutors(program, options, filename) {
  const found = [];
  const { factories, types } = scopedImports(program, options, filename);
  if (factories.size === 0 && types.size === 0) return found;
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
        for (const name of parameterNames(param, types)) {
          found.push({ name, range: node.range });
        }
      }
    }
    for (const child of childNodes(node)) walk(child, scope);
  };
  walk(program, program);
  return found;
}

function isScopedExecutor(scoped, name, node) {
  const position = node?.range?.[0];
  return Boolean(
    scoped?.some(
      (entry) => entry.name === name && entry.range[0] <= position && position < entry.range[1],
    ),
  );
}

module.exports = { collectScopedExecutors, isScopedExecutor };
