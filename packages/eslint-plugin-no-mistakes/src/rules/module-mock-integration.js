"use strict";

const { existsSync, readFileSync } = require("node:fs");
const { resolve } = require("node:path");
const { isInternalSpecifier, propertyName } = require("./module-mock-helpers");

function mockedExportNames(factory) {
  if (!factory) return null;
  const body = unwrapExpression(factory.type === "ObjectExpression" ? factory : factory.body);
  const objects = [];
  if (body?.type === "ObjectExpression") objects.push(body);
  if (body?.type === "BlockStatement") {
    for (const statement of body.body) collectReturnedObjects(statement, objects);
  }
  if (objects.length === 0) return null;
  if (objects.some((object) => !object)) return null;
  const names = [];
  for (const object of objects) {
    for (const prop of object.properties) {
      if (prop.type === "SpreadElement" || prop.computed) return null;
      const name = propertyName(prop.key);
      if (name && name !== "__esModule") names.push(name);
    }
  }
  return names.length > 0 ? names : null;
}

function unwrapExpression(node) {
  let current = node;
  while (
    current?.type === "TSAsExpression" ||
    current?.type === "TSSatisfiesExpression" ||
    current?.type === "TSNonNullExpression" ||
    current?.type === "TypeCastExpression"
  ) {
    current = current.expression;
  }
  return current;
}

function collectReturnedObjects(node, objects) {
  if (
    node.type === "FunctionDeclaration" ||
    node.type === "FunctionExpression" ||
    node.type === "ArrowFunctionExpression"
  ) {
    return;
  }
  if (node.type === "ReturnStatement") {
    const argument = unwrapExpression(node.argument);
    objects.push(argument?.type === "ObjectExpression" ? argument : null);
    return;
  }
  for (const key of ["block", "body", "consequent", "alternate", "finalizer", "handler"]) {
    const child = node[key];
    if (Array.isArray(child)) {
      for (const item of child) collectReturnedObjects(item, objects);
    } else if (child?.type) {
      collectReturnedObjects(child, objects);
    }
  }
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

function integrationExportNames(specifier, config) {
  const path = integrationSourcePath(specifier, config);
  if (!path) return null;
  const source = readFileSync(path, "utf8");
  const marker = config.markerRegex ?? String.raw`/\*\s*no-mistakes:\s*integration=[^*]+\*/`;
  const names = new Set();
  const declaration = safeRegExp(
    `${marker}\\s*export\\s+(?:async\\s+)?(?:function|const|let|var|class)\\s+([A-Za-z_$][\\w$]*)`,
    "g",
  );
  if (!declaration) return null;
  for (const match of source.matchAll(declaration)) names.add(match[1]);
  const defaultDeclaration = safeRegExp(
    `${marker}\\s*export\\s+default\\s+(?:async\\s+)?(?:function|class)\\b`,
    "g",
  );
  if (!defaultDeclaration) return null;
  for (const _match of source.matchAll(defaultDeclaration)) names.add("default");
  const named = safeRegExp(`${marker}\\s*export\\s*\\{([^}]+)\\}`, "g");
  if (!named) return null;
  for (const match of source.matchAll(named)) {
    for (const part of (match[1] ?? "").split(",")) {
      const exported = part
        .trim()
        .split(/\s+as\s+/)
        .pop()
        ?.trim();
      if (/^[A-Za-z_$][\w$]*$/.test(exported ?? "")) names.add(exported);
    }
  }
  return names;
}

function integrationAllows(specifier, factory, options) {
  const config = options.integrationExports;
  if (!config) return false;
  const specifierPatterns = config.specifiers ?? [];
  if (
    specifierPatterns.length > 0 &&
    !isInternalSpecifier(specifier, { internalSpecifiers: specifierPatterns })
  ) {
    return false;
  }
  const mocked = mockedExportNames(factory);
  if (!mocked) return false;
  const allowed = integrationExportNames(specifier, config);
  if (!allowed) return false;
  return mocked.every((name) => allowed.has(name));
}

module.exports = {
  integrationAllows,
};
